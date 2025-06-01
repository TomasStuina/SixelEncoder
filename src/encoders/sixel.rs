use std::{cmp, error::Error, io::{self, Write}, ops::Div};
use bmp::{Image, Pixel};


#[derive(PartialEq, PartialOrd)]
pub enum Color {
    RGB(u8, u8, u8),
    HSL(u16, u8, u8)
}

#[derive(PartialEq, PartialOrd)]
pub enum ColorMode {
    RGB,
    HSL
}

pub struct SixelEncoder
{
    color_palete: Vec<Color>,
    image_color_map: Vec<usize>,
    width: usize,
    height: usize,
}

impl SixelEncoder {
    
    pub fn new(image: &Image, color_mode: ColorMode) -> Self {
        let width = image.get_width() as usize;
        let height = image.get_height() as usize;
        let mut color_palete = vec![];
        let mut image_color_map = vec![0_usize; width * height];

        Self::generate_image_color_map(color_mode, image, &mut image_color_map, &mut color_palete);
        Self {
            color_palete,
            image_color_map,
            width,
            height
        }
    }

    pub fn encode<W: Write>(&mut self, writer: &mut io::BufWriter<W>) -> Result<(), Box<dyn Error>> {

        writeln!(writer, "{esc}Pq", esc = 27 as char)?;
        // writeln!(writer, "#0;2;0;0;0");

        for (i, color) in self.color_palete.iter().enumerate() {

            match color {
                Color::HSL(h, s, l) => writeln!(writer, "#{index};1;{h};{l};{s}", index = i, h = h, s = s, l = l)?,
                Color::RGB(r,g,b) => writeln!(writer, "#{index};2;{r};{g};{b}", index = i, r = r, g = g , b = b)?
            }
        }

        writeln!(writer, "\"1;1;{width};{height}", height = self.height, width = self.width)?;

        let mut y = 0;
        while y < self.height {

            for vertical_offset in 0..6 {
                if y + vertical_offset < self.height {
                   write!(writer, "$")?;

                    let line_start = ((y + vertical_offset) * self.width) as usize;
                    let line_end = line_start + self.width as usize;
                    Self::print_pixel_line(writer, &self.image_color_map[line_start..line_end], 1 << vertical_offset)?;
                }
            }

           write!(writer, "-")?;
            y += 5;
        }

        writeln!(writer, "{esc}\\", esc = 27 as char)?;

        Ok(())
    }

    fn generate_image_color_map(color_mode: ColorMode, image: &Image, image_color_map: &mut Vec<usize>, color_palete: &mut Vec<Color>) {
        for (x, y) in image.coordinates() {
            let pixel = image.get_pixel(x, y);
            let rgb = match color_mode {
                ColorMode::RGB => {
                    let (r, g, b) = Self::rgb_to_percents(pixel);
                    Color::RGB(r, g, b)
                },
                ColorMode::HSL => {
                    let (h, s, l) = Self::rgb_to_hsl(pixel);
                    Color::HSL(h, s, l)
                }
            };
            let coords = ((y * image.get_width()) + x) as usize;

            image_color_map[coords] = match color_palete.iter().position(|n| *n == rgb) {
                Some(color_index) =>  color_index,
                None => {
                    color_palete.push(rgb);
                    color_palete.len() - 1
                }
            };
        }
    }

    fn print_pixel_line<W: Write>(writer: &mut io::BufWriter<W>, line_color_map: &[usize], line_mask: u8) -> Result<(), Box<dyn Error>> {
        for color_index in line_color_map  {
           write!(writer, "#{color_index}{chr}", color_index = color_index, chr = (0x3Fu8 + line_mask) as char)?;
        }

        Ok(())
    }

    fn rgb_to_percents(pixel: Pixel) -> (u8, u8, u8) {
        (Self::channel_to_percents(pixel.r, 8),
        Self::channel_to_percents(pixel.g, 8),
        Self::channel_to_percents(pixel.b, 4))
    }

    fn rgb_to_hsl(pixel: Pixel) -> (u16, u8, u8) {

        let (r, g, b) = (pixel.r as f32 / 255_f32, pixel.g as f32 / 255_f32, pixel.b as f32 / 255_f32);

        let c_max = f32::max(f32::max(r, g), b) as f32;
        let c_min = f32::min(f32::min(r, g), b) as f32;
        let c_delta = c_max - c_min;

        let hue: f32 = if f32::abs(c_delta - 0.0) <= f32::EPSILON {
            0.0
        } else if c_max == r {
            (((g - b) / c_delta) % 6.0) * 60.0
        } else if c_max == g {
            (((b - r) / c_delta) + 2.0) * 60.0
        } else {
            (((r - g) / c_delta) + 4.0) * 60.0
        };

        let light = (c_max + c_min).div(2.0);

        let saturation= if f32::abs(c_delta - 0.0) <= f32::EPSILON
        {
            0.0
        } else {
            let divider = 1.0 - f32::abs((2.0 * light) - 1.0);
            
            c_delta / divider
        };

        (hue as u16, (saturation * 100.0) as u8, (light * 100.0) as u8)
    } 

    fn channel_to_percents(channel: u8, colors: u8) -> u8 {
        let colors_in_range: f32 = 256_f32 / colors as f32;
        let channel: i32 = channel as i32;
        let quantized_color = ((channel + 1) as f32 / colors_in_range).round();
        let percentage = (quantized_color / colors as f32) * 100_f32;

        percentage.floor() as u8
    } 
}

