use fontdue::Font;
use fontdue::Metrics;

#[derive(Copy, Clone)]
pub struct Glyph {
    pub x: f32,
    pub y: f32,
    pub metrics: Metrics,
}

pub struct GlyphAtlas {
    pub image: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub glyphs: Vec<Glyph>,
}

impl GlyphAtlas {
    pub fn new(font: Font, font_size: u32) -> Self {
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

        let n_glyphs = metrics.len();
        let n_glyphs_per_row = (n_glyphs as f32).sqrt().ceil() as usize;
        let image_height = max_glyph_height * n_glyphs_per_row;
        let image_width = max_glyph_width * n_glyphs_per_row;

        let mut image = vec![0u8; image_width * image_height];
        for i_glyph in 0..n_glyphs {
            let ir = (i_glyph / n_glyphs_per_row) * max_glyph_height;
            let ic = (i_glyph % n_glyphs_per_row) * max_glyph_width;
            let metric = &metrics[i_glyph];
            let glyph = Glyph {
                x: ic as f32,
                y: (image_height - ir) as f32,
                metrics: *metric,
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
            image: flipped_image,
            width: image_width as u32,
            height: image_height as u32,
            glyphs,
        }
    }

    pub fn get_glyph(&self, c: char) -> Glyph {
        let mut idx = c as usize;
        if idx < 32 || idx > 126 {
            idx = 63; // Question mark
        }

        self.glyphs[idx - 32]
    }
}
