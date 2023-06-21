use crate::common::Pivot;
use crate::shapes::Rectangle;
use fontdue;
use nalgebra::Vector2;

#[derive(Copy, Clone)]
pub struct Glyph {
    pub rect: Rectangle,
    pub texcoords: Rectangle,
    pub advance: Vector2<f32>,
}

pub struct GlyphAtlas {
    pub pixels: Vec<u8>,
    pub image_width: u32,
    pub image_height: u32,

    pub font_size: u32,
    pub glyph_ascent: f32,
    pub glyph_descent: f32,

    glyphs: Vec<Glyph>,
}

impl GlyphAtlas {
    pub fn new(font_bytes: &[u8], font_size: u32) -> Self {
        let font = fontdue::Font::from_bytes(
            font_bytes,
            fontdue::FontSettings::default(),
        )
        .unwrap();

        let ascent;
        let descent;
        if let Some(metrics) =
            font.horizontal_line_metrics(font_size as f32)
        {
            descent = metrics.descent;
            ascent = metrics.ascent;
        } else {
            descent = -0.25 * font_size as f32;
            ascent = 0.85 * font_size as f32;
        }

        let mut glyphs = vec![];

        let mut metrics = Vec::new();
        let mut bitmaps = Vec::new();
        let mut max_glyph_width = 0;
        let mut max_glyph_height = 0;

        for u in 32..127 {
            let ch = char::from_u32(u).unwrap();

            let (metric, bitmap) = font.rasterize(ch, font_size as f32);

            assert!(bitmap.len() == metric.width * metric.height);

            metrics.push(metric);
            bitmaps.push(bitmap);

            max_glyph_width = max_glyph_width.max(metric.width);
            max_glyph_height = max_glyph_height.max(metric.height);
        }

        max_glyph_width += 2;
        max_glyph_height += 2;

        let n_glyphs = metrics.len();
        let n_glyphs_per_row = (n_glyphs as f32).sqrt().ceil() as usize;
        let image_height = max_glyph_height * n_glyphs_per_row;
        let image_width = max_glyph_width * n_glyphs_per_row;

        let mut image = vec![0u8; image_width * image_height];
        for i_glyph in 0..n_glyphs {
            let ir = (i_glyph / n_glyphs_per_row) * max_glyph_height;
            let ic = (i_glyph % n_glyphs_per_row) * max_glyph_width;
            let metric = &metrics[i_glyph];
            let texcoords = Rectangle::from_top_left(
                Vector2::new(ic as f32, (image_height - ir) as f32),
                Vector2::new(metric.width as f32, metric.height as f32),
            );
            let offset =
                Vector2::new(metric.xmin as f32, metric.ymin as f32);
            let size =
                Vector2::new(metric.width as f32, metric.height as f32);
            let advance =
                Vector2::new(metric.advance_width, metric.advance_height);
            let rect = Rectangle::from_bot_left(offset, size);

            let glyph = Glyph {
                texcoords,
                rect,
                advance,
            };
            glyphs.push(glyph);

            let bitmap = &bitmaps[i_glyph];
            assert!(bitmap.len() == metric.width * metric.height);

            for gr in 0..metric.height {
                let start = gr * metric.width;
                let end = start + metric.width;
                let glyph_row = &bitmap[start..end];

                let start = (ir + gr) * image_width + ic;
                let end = start + metric.width;
                image[start..end].copy_from_slice(&glyph_row);
            }
        }

        let mut flipped_image = vec![0u8; image_width * image_height];
        for r in 0..image_height {
            let start = (image_height - r - 1) * image_width;
            let end = start + image_width;
            let source = &image[start..end];

            let start = r * image_width;
            let end = start + image_width;
            flipped_image[start..end].copy_from_slice(source);
        }

        Self {
            pixels: flipped_image,
            image_width: image_width as u32,
            image_height: image_height as u32,
            font_size,
            glyph_ascent: ascent,
            glyph_descent: descent,
            glyphs,
        }
    }

    fn get_text_size(&self, text: &str) -> Vector2<f32> {
        let mut width: f32 = 0.0;
        let mut height: f32 = 0.0;
        for symbol in text.chars() {
            let glyph = self.get_glyph(symbol);
            width += glyph.advance.x;
            height = height.max(glyph.rect.get_height());
        }

        Vector2::new(width, height)
    }

    pub fn iter_text_glyphs<'a>(
        &'a self,
        pivot: Pivot,
        text: &'a str,
    ) -> impl IntoIterator<Item = Glyph> + 'a {
        use Pivot::*;

        let size = self.get_text_size(text);
        let mut cursor = match pivot {
            BotLeft(pos) => pos,
            Center(pos) => pos - size * 0.5,
            TopCenter(mut pos) => {
                pos.x -= size.x * 0.5;
                pos.y -= size.y;
                pos
            }
            BotCenter(mut pos) => {
                pos.x -= size.x * 0.5;
                pos
            }
            _ => {
                todo!()
            }
        };

        text.chars().map(move |symbol| {
            let mut glyph = self.get_glyph(symbol);
            glyph.rect.translate_assign(&cursor);
            cursor += glyph.advance;

            glyph
        })
    }

    fn get_glyph(&self, symbol: char) -> Glyph {
        let mut idx = symbol as usize;
        if idx < 32 || idx > 126 {
            idx = 63; // Question mark
        }

        let glyph = self.glyphs[idx - 32];

        glyph
    }
}
