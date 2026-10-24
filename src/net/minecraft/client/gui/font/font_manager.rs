//! net/minecraft/client/gui/font/FontManager.java
//!
//! The fonts the resource packs define (`font/*.json`): each file's providers, bottom pack first
//! and each file's last provider first, the ones that load a provider loaded once and the
//! references resolved to the referenced fonts' providers (their filters merged), then each font
//! made a `FontSet` of its providers in the files' order with the fallback last. Java loads on
//! its executor and resolves in `DependencySorter`'s order; resolving a font when it's first
//! referenced gives the same providers, and a reference in a cycle or to a font that didn't load
//! leaves its font out, like Java's. A provider that fails to load is logged and rejected, which
//! leaves out every font that has it.
//!
//! Not ported yet: the atlases' and the players' sprite fonts (`AtlasGlyphProvider`,
//! `PlayerGlyphProvider`, a `FontDescription` that isn't a resource draws the missing font), the
//! last font a provider handed out (`CachedFontProvider.lastEntry`, which only saves a lookup),
//! and changing the font options without a reload (`updateOptions`).

use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;

use crate::com::mojang::blaze3d::font::glyph_provider::{Conditional, GlyphProvider};
use crate::com::mojang::renderpearl::api::device::gpu_device::GpuDevice;
use crate::net::minecraft::client::gui::font::all_missing_glyph_provider::AllMissingGlyphProvider;
use crate::net::minecraft::client::gui::font::font_option::{Filter, FontOption};
use crate::net::minecraft::client::gui::font::font_set::FontSet;
use crate::net::minecraft::client::gui::font::glyph_stitcher::GlyphStitcher;
use crate::net::minecraft::client::gui::font::providers::glyph_provider_definition::{
    self, Unpacked,
};
use crate::net::minecraft::network::chat::font_description::FontDescription;
use crate::net::minecraft::resources::file_to_id_converter::FileToIdConverter;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::server::packs::resources::resource::Resource;
use crate::net::minecraft::server::packs::resources::resource_manager::ResourceManager;

pub struct FontManager {
    missing_font_set: Rc<RefCell<FontSet>>,
    font_sets: HashMap<Identifier, Rc<RefCell<FontSet>>>,
}

/// `FontManager.BuilderId`.
#[derive(Clone, Debug)]
struct BuilderId {
    font_id: Identifier,
    pack: String,
    index: usize,
}

impl std::fmt::Display for BuilderId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({}: builder #{} from pack {})",
            self.font_id, self.index, self.pack
        )
    }
}

/// `FontManager.BuilderResult`: a loaded provider (none where it failed) or a reference.
enum BuilderResult {
    Provider(Option<Rc<dyn GlyphProvider>>, Filter),
    Reference(Identifier, Filter),
}

impl FontManager {
    pub const MISSING_FONT: &str = "missing";

    /// `Minecraft.DEFAULT_FONT`.
    pub fn default_font() -> Identifier {
        Identifier::with_default_namespace("default")
    }

    /// The fonts the resources define, their pages on `device` (none for layouts alone), with
    /// the font options (`forceUnicodeFont`, `japaneseGlyphVariants`).
    pub fn load(
        device: Option<&'static dyn GpuDevice>,
        manager: &dyn ResourceManager,
        options: &[FontOption],
    ) -> FontManager {
        let missing_font_set = FontManager::create_font_set(
            device,
            &Identifier::with_default_namespace(FontManager::MISSING_FONT),
            vec![FontManager::create_fallback_provider()],
            &[],
        );

        // prepare: every font's builders, loaded.
        let font_definitions = FileToIdConverter::json("font");
        let mut bundles: BTreeMap<Identifier, Vec<BuilderResult>> = BTreeMap::new();
        for (file, resources) in font_definitions.list_matching_resource_stacks(manager) {
            let font_name = font_definitions.file_to_id(&file);
            let mut builders = Vec::new();
            for (id, conditional) in FontManager::load_resource_stack(&resources, &font_name) {
                let filter = conditional.filter.clone();
                builders.push(match conditional.definition.unpack() {
                    Unpacked::Loader(definition) => {
                        let provider = match definition.load(manager) {
                            Ok(provider) => Some(provider),
                            Err(error) => {
                                eprintln!("Failed to load builder {id}, rejecting: {error}");
                                None
                            }
                        };
                        BuilderResult::Provider(provider, filter)
                    }
                    Unpacked::Reference(reference) => BuilderResult::Reference(reference, filter),
                });
            }
            bundles.insert(font_name, builders);
        }

        // resolveProviders, then apply: each font's providers in the files' order, the fallback
        // last.
        let mut resolved: HashMap<Identifier, Option<Vec<Conditional>>> = HashMap::new();
        let ids: Vec<Identifier> = bundles.keys().cloned().collect();
        for id in &ids {
            FontManager::resolve(id, &bundles, &mut resolved, &mut Vec::new());
        }
        let mut font_sets = HashMap::new();
        for id in ids {
            if let Some(Some(mut providers)) = resolved.remove(&id) {
                // finalizeProviderLoading puts the fallback first, which the reverse puts last.
                providers.insert(0, FontManager::create_fallback_provider());
                providers.reverse();
                let font_set = FontManager::create_font_set(device, &id, providers, options);
                font_sets.insert(id, font_set);
            }
        }
        if !font_sets.contains_key(&FontManager::default_font()) {
            panic!("Default font failed to load");
        }
        FontManager {
            missing_font_set,
            font_sets,
        }
    }

    fn create_font_set(
        device: Option<&'static dyn GpuDevice>,
        id: &Identifier,
        providers: Vec<Conditional>,
        options: &[FontOption],
    ) -> Rc<RefCell<FontSet>> {
        let stitcher = GlyphStitcher::new(device, id.clone());
        let mut result = FontSet::new(stitcher);
        result.reload_providers(providers, options);
        Rc::new(RefCell::new(result))
    }

    fn create_fallback_provider() -> Conditional {
        Conditional {
            provider: Rc::new(AllMissingGlyphProvider::new()),
            filter: Filter::ALWAYS_PASS,
        }
    }

    /// A font's providers in its builders' order, its references' in their place; none when a
    /// builder failed, a reference is missing, or the font is in a cycle.
    fn resolve(
        id: &Identifier,
        bundles: &BTreeMap<Identifier, Vec<BuilderResult>>,
        resolved: &mut HashMap<Identifier, Option<Vec<Conditional>>>,
        visiting: &mut Vec<Identifier>,
    ) -> Option<Vec<Conditional>> {
        if let Some(result) = resolved.get(id) {
            return result.clone();
        }
        let builders = bundles.get(id)?;
        if visiting.contains(id) {
            return None;
        }
        visiting.push(id.clone());
        let mut result = Some(Vec::new());
        for builder in builders {
            let resolved_builder = match builder {
                BuilderResult::Provider(provider, filter) => provider.as_ref().map(|provider| {
                    vec![Conditional {
                        provider: Rc::clone(provider),
                        filter: filter.clone(),
                    }]
                }),
                BuilderResult::Reference(reference, filter) => {
                    match FontManager::resolve(reference, bundles, resolved, visiting) {
                        Some(references) => Some(
                            references
                                .into_iter()
                                .map(|original| Conditional {
                                    filter: filter.merge(&original.filter),
                                    provider: original.provider,
                                })
                                .collect(),
                        ),
                        None => {
                            eprintln!(
                                "Can't find font {reference} referenced by a builder of {id}, either because it's missing, failed to load or is part of loading cycle"
                            );
                            None
                        }
                    }
                }
            };
            match (resolved_builder, &mut result) {
                (Some(resolved_builder), Some(result)) => result.extend(resolved_builder),
                _ => {
                    result = None;
                    break;
                }
            }
        }
        visiting.pop();
        resolved.insert(id.clone(), result.clone());
        result
    }

    /// `loadResourceStack`: each resource's providers, last first; a file that doesn't decode is
    /// logged and skipped.
    fn load_resource_stack(
        resource_stack: &[Resource],
        font_name: &Identifier,
    ) -> Vec<(BuilderId, glyph_provider_definition::Conditional)> {
        let mut builder_stack = Vec::new();
        for resource in resource_stack {
            let providers = resource
                .open()
                .map_err(|error| error.to_string())
                .and_then(|bytes| {
                    serde_json::from_slice::<serde_json::Value>(&bytes)
                        .map_err(|error| error.to_string())
                })
                .and_then(|json| {
                    json.get("providers")
                        .and_then(|providers| providers.as_array())
                        .ok_or_else(|| "No key providers in MapLike".to_owned())?
                        .iter()
                        .map(glyph_provider_definition::Conditional::decode)
                        .collect::<Result<Vec<_>, String>>()
                });
            match providers {
                Ok(providers) => {
                    for (index, provider) in providers.into_iter().enumerate().rev() {
                        let id = BuilderId {
                            font_id: font_name.clone(),
                            pack: resource.source_pack_id().to_owned(),
                            index,
                        };
                        builder_stack.push((id, provider));
                    }
                }
                Err(error) => eprintln!(
                    "Unable to load font '{font_name}' in fonts.json in resourcepack: '{}': {error}",
                    resource.source_pack_id()
                ),
            }
        }
        builder_stack
    }

    /// `getFontSetRaw`: the font, or the missing font.
    pub fn get_font_set_raw(&self, id: &Identifier) -> Rc<RefCell<FontSet>> {
        Rc::clone(self.font_sets.get(id).unwrap_or(&self.missing_font_set))
    }

    /// `CachedFontProvider.getGlyphSource`: a resource font, or the missing font for the sprite
    /// fonts (an atlas's, a player's head), which aren't ported.
    pub fn glyph_source(&self, description: &FontDescription, non_fishy_only: bool) -> GlyphSource {
        let font_set = match description {
            FontDescription::Resource { id } => self.get_font_set_raw(id),
            FontDescription::AtlasSprite { .. } | FontDescription::PlayerSprite { .. } => {
                Rc::clone(&self.missing_font_set)
            }
        };
        GlyphSource {
            font_set,
            filter_fishy_glyphs: non_fishy_only,
        }
    }
}

/// `GlyphSource`, as a font set's `Source`: its glyphs, with or without fishy advances.
#[derive(Clone)]
pub struct GlyphSource {
    pub font_set: Rc<RefCell<FontSet>>,
    pub filter_fishy_glyphs: bool,
}

impl GlyphSource {
    pub fn get_glyph(
        &self,
        codepoint: i32,
    ) -> Rc<crate::net::minecraft::client::gui::font::glyphs::baked_glyph::BakedGlyph> {
        self.font_set
            .borrow_mut()
            .get_glyph(codepoint, self.filter_fishy_glyphs)
    }

    pub fn get_random_glyph(
        &self,
        random: &mut dyn crate::net::minecraft::util::random_source::RandomSource,
        width: i32,
    ) -> Rc<crate::net::minecraft::client::gui::font::glyphs::baked_glyph::BakedGlyph> {
        self.font_set.borrow_mut().get_random_glyph(random, width)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;
    use crate::com::mojang::blaze3d::vertex::buffer_builder::BufferBuilder;
    use crate::com::mojang::blaze3d::vertex::byte_buffer_builder::ByteBufferBuilder;
    use crate::golden::Encode;
    use crate::golden::{Golden, hex};
    use crate::net::minecraft::client::gui::font::Font;
    use crate::net::minecraft::client::gui::font::glyphs::baked_glyph::BakedGlyph;
    use crate::net::minecraft::client::gui::font::text_renderable::TextRenderable;
    use crate::net::minecraft::client::gui::navigation::screen_rectangle::ScreenRectangle;
    use crate::net::minecraft::client::string_splitter::StringSplitter;
    use crate::net::minecraft::network::chat::formatted_text::OfStyled;
    use crate::net::minecraft::network::chat::style::Style;
    use crate::net::minecraft::server::packs::resources::multi_pack_resource_manager::tests::vanilla_resources;
    use crate::org::joml::matrix4f::Matrix4f;
    use serde_json::{Value, json};

    fn bits(value: f32) -> Value {
        Value::from(format!("{:08x}", value.to_bits()))
    }

    fn style(style: &Style) -> Value {
        json!([
            style.is_bold(),
            style.is_italic(),
            style.is_underlined(),
            style.is_strikethrough(),
            style.is_obfuscated(),
            style.get_color().map(|color| color.get_value())
        ])
    }

    /// `FontSuites.glyph`: the metrics, and a sheet glyph's UVs, place and the pages so far.
    fn glyph(manager: &FontManager, font: &str, codepoint: i32, filter_fishy: bool) -> Value {
        let font_set = manager.get_font_set_raw(&Identifier::parse(font));
        let glyph = font_set.borrow_mut().get_glyph(codepoint, filter_fishy);
        let info = glyph.info();
        let mut result = vec![
            bits(info.get_advance_bold(false)),
            bits(info.get_advance_bold(true)),
            bits(info.get_bold_offset()),
            bits(info.get_shadow_offset()),
        ];
        if let BakedGlyph::Sheet(sheet) = &*glyph {
            for value in [
                sheet.u0,
                sheet.u1,
                sheet.v0,
                sheet.v1,
                sheet.left,
                sheet.right,
                sheet.up,
                sheet.down,
            ] {
                result.push(bits(value));
            }
            result.push(Value::from(font_set.borrow().stitcher().texture_count()));
        }
        Value::Array(result)
    }

    /// The renderable's vertices, as `GlyphRenderState` builds them: flat, fully lit, unposed.
    fn vertices(renderable: &TextRenderable) -> Value {
        let pipeline = renderable.gui_pipeline();
        let mut buffer = ByteBufferBuilder::new(1024);
        let mut builder = BufferBuilder::new(
            &mut buffer,
            pipeline.get_primitive_topology(),
            pipeline
                .get_vertex_format_binding(0)
                .expect("a vertex format"),
        );
        renderable.render(&Matrix4f::new(), &mut builder, 15728880, true);
        match builder.build() {
            Some(mesh) => {
                let bytes: Vec<i8> = mesh
                    .vertex_buffer()
                    .iter()
                    .map(|&byte| byte as i8)
                    .collect();
                Value::from(hex(&bytes))
            }
            None => Value::Null,
        }
    }

    /// The vanilla pack's fonts, every glyph their providers define baked in vanilla's order (so
    /// the pages' layout), the widths, `prepareText`'s glyphs and effects, and the splitter's
    /// cuts (TESTING.md, "The font").
    #[test]
    fn lays_out_the_fonts_like_vanilla() {
        let golden = Golden::load("net.minecraft.client.gui.font.FontManager");
        let manager = Rc::new(FontManager::load(None, &vanilla_resources(), &[]));
        let mut fonts: Vec<String> = manager.font_sets.keys().map(|id| id.to_string()).collect();
        fonts.sort();
        golden.field("fonts", fonts);
        golden.check(
            "glyph",
            |font: String, codepoint: i32, filter_fishy: bool| {
                glyph(&manager, &font, codepoint, filter_fishy)
            },
        );

        let font = Font::new(Rc::clone(&manager), false);
        golden.check("width", |string: String| font.width(&string));
        golden.check("widthText", |string: String| {
            font.width_text(&OfStyled(string, Style::EMPTY.with_bold(Some(true))))
        });
        golden.check(
            "prepareText",
            |string: String, x: i32, y: i32, color: i32, shadow: bool, background: i32| {
                let prepared =
                    font.prepare_text_str(&string, x as f32, y as f32, color, shadow, background);
                let mut renderables = Vec::new();
                prepared.visit(&mut |renderable| {
                    renderables.push(json!([
                        match renderable {
                            TextRenderable::Glyph(_) => "glyph",
                            TextRenderable::Effect(_) => "effect",
                        },
                        bits(renderable.left()),
                        bits(renderable.top()),
                        bits(renderable.right()),
                        bits(renderable.bottom()),
                        renderable.gui_pipeline().get_location().to_string(),
                        vertices(renderable)
                    ]));
                });
                let bounds: Option<ScreenRectangle> = prepared.bounds();
                json!([renderables, bounds.encode()])
            },
        );

        let splitter = font.get_splitter();
        golden.check("splitLines", |string: String, width: i32| {
            let lines: Vec<Value> = splitter
                .split_lines(&string, width, &Style::EMPTY)
                .iter()
                .map(|(text, line_style)| json!([text, style(line_style)]))
                .collect();
            Value::Array(lines)
        });
        golden.check("plainIndexAtWidth", |string: String, width: i32| {
            splitter.plain_index_at_width(&string, width, &Style::EMPTY)
        });
        golden.check("plainTailByWidth", |string: String, width: i32| {
            splitter.plain_tail_by_width(&string, width, &Style::EMPTY)
        });
        golden.check("findLineBreak", |string: String, width: i32| {
            splitter.find_line_break(&string, width, &Style::EMPTY)
        });
        golden.check(
            "getWordPosition",
            |string: String, dir: i32, from: i32, strip: bool| {
                StringSplitter::get_word_position(&string, dir, from, strip)
            },
        );
        golden.finish();
    }
}
