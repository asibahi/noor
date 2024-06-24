use crate::{logic::VariationKind, Variation};

#[allow(dead_code)]
pub(crate) struct GlyphData {
    pub codepoint: u32,
    pub cluster: u32,
    pub x_advance: i32,
    pub y_advance: i32,
    pub x_offset: i32,
    pub y_offset: i32,
}

pub trait Shaper<'f> {
    fn load_font(font_data: &'f [u8]) -> Self;

    fn shape_text(&mut self, input: &str, variations: &[Variation]) -> Vec<GlyphData>;
}

pub(crate) struct HarfBuzz<'f>(harfbuzz_rs::Owned<harfbuzz_rs::Font<'f>>);

impl<'f> Shaper<'f> for HarfBuzz<'f> {
    fn load_font(font_data: &'f [u8]) -> Self {
        Self(harfbuzz_rs::Font::new(harfbuzz_rs::Face::from_bytes(font_data, 0)))
    }

    fn shape_text(&mut self, input: &str, variations: &[Variation]) -> Vec<GlyphData> {
        let buffer = harfbuzz_rs::UnicodeBuffer::new().add_str(input);
        self.0.set_variations(
            &variations
                .iter()
                .filter_map(|v| match v.kind {
                    VariationKind::Axis(tag) => {
                        Some(harfbuzz_rs::Variation::new(&tag, v.current_value))
                    }
                    VariationKind::Spacing => None,
                })
                .collect::<Vec<_>>(),
        );

        let output = harfbuzz_rs::shape(&self.0, buffer, &[]);

        let space = self.0.get_nominal_glyph(' ').unwrap();
        let space_width = self.0.get_glyph_h_advance(space);
        let space_width = match variations.iter().find(|v| matches!(v.kind, VariationKind::Spacing))
        {
            Some(v) => (space_width as f32 * v.current_value) as i32,
            None => space_width,
        };

        output
            .get_glyph_infos()
            .iter()
            .zip(output.get_glyph_positions())
            .map(|(i, p)| GlyphData {
                codepoint: i.codepoint,
                cluster: i.cluster,
                x_advance: if i.codepoint == space { space_width } else { p.x_advance },
                y_advance: p.y_advance,
                x_offset: p.x_offset,
                y_offset: p.y_offset,
            })
            .collect()
    }
}
