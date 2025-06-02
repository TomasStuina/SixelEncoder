use bmp::{Image, Pixel};

pub struct SixelEncoder
{
    color_palete: Vec<(u8, u8, u8)>,
    image_color_map: Vec<usize>,
    width: usize,
    height: usize
}

impl SixelEncoder {
    
    pub fn new(image: &Image) -> Self {
        let width = image.get_width() as usize;
        let height = image.get_height() as usize;
        let mut color_palete = vec![];
        let mut image_color_map = vec![0_usize; width * height];

        Self::generate_image_color_map(image, &mut image_color_map, &mut color_palete);
        Self {
            color_palete,
            image_color_map,
            width,
            height
        }
    }

    pub fn encode(&mut self) {
        println!("{esc}Pq", esc = 27 as char);
        // println!("#0;2;0;0;0");

        for (i, (r,g,b)) in self.color_palete.iter().enumerate() {
            println!("#{index};2;{r};{g};{b}", index = i, r = r, g = g , b = b);
        }

        println!("\"1;1;{height};{width}", height = self.height, width = self.width);

        let mut y = 0;
        while y < self.height {

            for vertical_offset in 0..6 {
                if y + vertical_offset < self.height {
                    print!("$");

                    let line_start = ((y + vertical_offset) * self.width) as usize;
                    let line_end = line_start + self.width as usize;
                    Self::print_pixel_line(&self.image_color_map[line_start..line_end], 1 << vertical_offset);
                }
            }

            print!("-");
            y += 5;
        }

        println!("{esc}\\", esc = 27 as char);
    }

    fn generate_image_color_map(image: &Image, image_color_map: &mut Vec<usize>, color_palete: &mut Vec<(u8, u8, u8)>) {
        for (x, y) in image.coordinates() {
            let pixel = image.get_pixel(x, y);
            let rgb = Self::rgb_to_percents(pixel);
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

    fn print_pixel_line(line_color_map: &[usize], line_mask: u8) {
        for color_index in line_color_map  {
            print!("#{color_index}{chr}", color_index = color_index, chr = (0x3Fu8 + line_mask) as char);
        }
    }

    fn rgb_to_percents(pixel: Pixel) -> (u8, u8, u8) {
        (Self::channel_to_percents(pixel.r), Self::channel_to_percents(pixel.g), Self::channel_to_percents(pixel.b))
    } 

    fn channel_to_percents(channel: u8) -> u8 {
        const TOTAL_CHANNEL_RANGES: f32 = 6_f32;
        const CHANNEL_RANGE_LENGTH: f32 = 256_f32 / TOTAL_CHANNEL_RANGES;

        let channel: i32 = channel as i32;
        let smoothed_color = ((channel + 1) as f32 / CHANNEL_RANGE_LENGTH).round();
        let percentage = (smoothed_color / TOTAL_CHANNEL_RANGES) * 100_f32;

        percentage.floor() as u8
    } 
}

