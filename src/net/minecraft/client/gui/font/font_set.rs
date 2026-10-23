//! net/minecraft/client/gui/font/FontSet.java
//!
//! A font: its providers (those its options select, and of those the ones that have a glyph some
//! codepoint is drawn with), the glyph each codepoint is drawn with (the first provider's that has
//! one, and the first whose advance isn't fishy for text that filters those), and its pages.
//! Glyphs are baked into the pages when first asked for, the missing and white glyphs first, so a
//! page's layout is the order the font's glyphs were first used in. Java's `DelayedBake`s are each
//! codepoint's chosen providers, whose bakes are kept here.
//!
//! The lists of glyphs by width (`glyphsByWidth`), which obfuscated text draws random glyphs from,
//! are in codepoint order here and in a fastutil hash set's in Java, so obfuscated text isn't the
//! same yet.

// Java's negated comparisons are kept as they are: they differ from the plain ones for NaN.
#![allow(clippy::neg_cmp_op_on_partial_ord)]

use std::collections::BTreeMap;
use std::rc::Rc;

use crate::com::mojang::blaze3d::font::glyph_bitmap::GlyphBitmap;
use crate::com::mojang::blaze3d::font::glyph_info::GlyphInfo;
use crate::com::mojang::blaze3d::font::glyph_provider::{Conditional, GlyphProvider};
use crate::com::mojang::blaze3d::font::unbaked_glyph::{Stitcher, UnbakedGlyph};
use crate::net::minecraft::client::gui::font::codepoint_map::CodepointMap;
use crate::net::minecraft::client::gui::font::font_option::FontOption;
use crate::net::minecraft::client::gui::font::glyph_stitcher::GlyphStitcher;
use crate::net::minecraft::client::gui::font::glyphs::baked_glyph::BakedGlyph;
use crate::net::minecraft::client::gui::font::glyphs::baked_sheet_glyph::BakedSheetGlyph;
use crate::net::minecraft::client::gui::font::glyphs::special_glyphs::SpecialGlyphs;
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::util::random_source::RandomSource;

pub struct FontSet {
    stitcher: GlyphStitcher,
    all_providers: Vec<Conditional>,
    active_providers: Vec<Rc<dyn GlyphProvider>>,
    glyphs_by_width: BTreeMap<i32, Vec<i32>>,
    glyph_cache: CodepointMap<SelectedGlyphs>,
    missing_glyph: Rc<BakedGlyph>,
    white_glyph: Option<Rc<BakedSheetGlyph>>,
}

/// `FontSet.DelayedBake`: a provider's glyph, baked when first asked for.
#[derive(Clone)]
struct DelayedBake {
    unbaked: Rc<dyn UnbakedGlyph>,
    baked: std::cell::RefCell<Option<Rc<BakedGlyph>>>,
}

/// `FontSet.SelectedGlyphs`: the glyph drawn for any text, and for text that filters fishy
/// advances. The non-fishy one is the same bake when it's the same glyph, or the missing glyph.
#[derive(Clone)]
enum SelectedGlyphs {
    Missing,
    Selected {
        any: Rc<DelayedBake>,
        non_fishy: Option<Rc<DelayedBake>>,
    },
}

/// `FontSet.wrappedStitcher`: the font's pages, with the missing glyph for what doesn't fit.
struct WrappedStitcher<'a> {
    stitcher: &'a mut GlyphStitcher,
    missing_glyph: &'a Rc<BakedGlyph>,
}

impl Stitcher for WrappedStitcher<'_> {
    fn stitch(&mut self, info: GlyphInfo, glyph_bitmap: &dyn GlyphBitmap) -> Rc<BakedGlyph> {
        match self.stitcher.stitch(info, glyph_bitmap) {
            Some(glyph) => Rc::new(BakedGlyph::Sheet(glyph)),
            None => Rc::clone(self.missing_glyph),
        }
    }

    fn get_missing(&self) -> Rc<BakedGlyph> {
        Rc::clone(self.missing_glyph)
    }
}

impl FontSet {
    const LARGE_FORWARD_ADVANCE: f32 = 32.0;

    pub fn new(stitcher: GlyphStitcher) -> FontSet {
        FontSet {
            stitcher,
            all_providers: Vec::new(),
            active_providers: Vec::new(),
            glyphs_by_width: BTreeMap::new(),
            glyph_cache: CodepointMap::new(),
            missing_glyph: Rc::new(BakedGlyph::InvisibleMissing),
            white_glyph: None,
        }
    }

    pub fn reload_providers(&mut self, providers: Vec<Conditional>, options: &[FontOption]) {
        self.all_providers = providers;
        self.reload(options);
    }

    pub fn reload(&mut self, options: &[FontOption]) {
        self.active_providers = Vec::new();
        self.reset_textures();
        self.active_providers = self.select_providers(options);
    }

    fn reset_textures(&mut self) {
        self.stitcher.reset();
        self.glyph_cache.clear();
        self.glyphs_by_width.clear();
        self.missing_glyph = Rc::new(BakedGlyph::Sheet(
            SpecialGlyphs::MISSING
                .bake(&mut self.stitcher)
                .expect("the missing glyph fits an empty page"),
        ));
        self.white_glyph = SpecialGlyphs::WHITE.bake(&mut self.stitcher);
    }

    fn select_providers(&mut self, options: &[FontOption]) -> Vec<Rc<dyn GlyphProvider>> {
        let mut supported_glyphs = std::collections::BTreeSet::new();
        let mut selected_providers: Vec<Rc<dyn GlyphProvider>> = Vec::new();
        for conditional_provider in &self.all_providers {
            if conditional_provider.filter.apply(options) {
                selected_providers.push(Rc::clone(&conditional_provider.provider));
                supported_glyphs.extend(conditional_provider.provider.get_supported_glyphs());
            }
        }

        let mut used_providers = vec![false; selected_providers.len()];
        for codepoint in supported_glyphs {
            for (index, provider) in selected_providers.iter().enumerate() {
                if let Some(glyph) = provider.get_glyph(codepoint) {
                    used_providers[index] = true;
                    let info = glyph.info();
                    if info != GlyphInfo::Special(SpecialGlyphs::MISSING) {
                        self.glyphs_by_width
                            .entry(Mth::ceil_f32(info.get_advance_bold(false)))
                            .or_default()
                            .push(codepoint);
                    }
                    break;
                }
            }
        }
        selected_providers
            .into_iter()
            .zip(used_providers)
            .filter_map(|(provider, used)| used.then_some(provider))
            .collect()
    }

    fn has_fishy_advance(glyph: &GlyphInfo) -> bool {
        let advance = glyph.get_advance_bold(false);
        if !(advance < 0.0) && !(advance > FontSet::LARGE_FORWARD_ADVANCE) {
            let bold_advance = glyph.get_advance_bold(true);
            bold_advance < 0.0 || bold_advance > FontSet::LARGE_FORWARD_ADVANCE
        } else {
            true
        }
    }

    fn compute_glyph_info(&self, codepoint: i32) -> SelectedGlyphs {
        let mut first_glyph: Option<Rc<DelayedBake>> = None;
        for provider in &self.active_providers {
            if let Some(glyph) = provider.get_glyph(codepoint) {
                let first = first_glyph.get_or_insert_with(|| {
                    Rc::new(DelayedBake {
                        unbaked: Rc::clone(&glyph),
                        baked: std::cell::RefCell::new(None),
                    })
                });
                if !FontSet::has_fishy_advance(&glyph.info()) {
                    if Rc::ptr_eq(&first.unbaked, &glyph) {
                        return SelectedGlyphs::Selected {
                            any: Rc::clone(first),
                            non_fishy: Some(Rc::clone(first)),
                        };
                    }
                    return SelectedGlyphs::Selected {
                        any: Rc::clone(first),
                        non_fishy: Some(Rc::new(DelayedBake {
                            unbaked: glyph,
                            baked: std::cell::RefCell::new(None),
                        })),
                    };
                }
            }
        }
        match first_glyph {
            Some(first) => SelectedGlyphs::Selected {
                any: first,
                non_fishy: None,
            },
            None => SelectedGlyphs::Missing,
        }
    }

    /// `getGlyph(codepoint).select(filterFishy).get()`: the glyph, baked if it's the first time.
    pub fn get_glyph(&mut self, codepoint: i32, filter_fishy: bool) -> Rc<BakedGlyph> {
        if self.glyph_cache.get(codepoint).is_none() {
            let selected = self.compute_glyph_info(codepoint);
            self.glyph_cache.put(codepoint, selected);
        }
        let selected = self
            .glyph_cache
            .get(codepoint)
            .expect("just cached")
            .clone();
        let delayed = match selected {
            SelectedGlyphs::Missing => return Rc::clone(&self.missing_glyph),
            SelectedGlyphs::Selected { any, non_fishy } => {
                if filter_fishy {
                    match non_fishy {
                        Some(non_fishy) => non_fishy,
                        None => return Rc::clone(&self.missing_glyph),
                    }
                } else {
                    any
                }
            }
        };
        if let Some(baked) = &*delayed.baked.borrow() {
            return Rc::clone(baked);
        }
        let baked = delayed.unbaked.bake(&mut WrappedStitcher {
            stitcher: &mut self.stitcher,
            missing_glyph: &self.missing_glyph,
        });
        *delayed.baked.borrow_mut() = Some(Rc::clone(&baked));
        baked
    }

    /// `getRandomGlyph`: a non-fishy glyph of the width, or the missing glyph.
    pub fn get_random_glyph(
        &mut self,
        random: &mut dyn RandomSource,
        width: i32,
    ) -> Rc<BakedGlyph> {
        let codepoint = match self.glyphs_by_width.get(&width) {
            Some(chars) if !chars.is_empty() => {
                chars[random.next_int_bounded(chars.len() as i32) as usize]
            }
            _ => return Rc::clone(&self.missing_glyph),
        };
        self.get_glyph(codepoint, true)
    }

    /// `whiteGlyph()`: panics before a reload, where Java's `requireNonNull` throws.
    pub fn white_glyph(&self) -> Rc<BakedSheetGlyph> {
        Rc::clone(self.white_glyph.as_ref().expect("the white glyph"))
    }

    /// The pages so far, for the goldens.
    pub fn stitcher(&self) -> &GlyphStitcher {
        &self.stitcher
    }
}
