use std::fs;
use std::env;
use std::io::BufWriter;

use printpdf::*;

extern crate regex;
use regex::Regex;

struct Text {
    font: String,
    word: String,
}

struct TextLine {
    children: Vec<Text>,
}

struct Thing {
    header: String,
    color: [f64; 3],
    children: Vec<TextLine>,
}

fn parse_logs(filename: String) -> Vec<Thing> {
    let contents = fs::read_to_string(filename)
        .expect("Baah");

    let bold_re = Regex::new(r"(\*\*|__)(.+)(\*\*|__)").unwrap();
    let italic_re = Regex::new(r"(\*|_)(.+)(\*|_)").unwrap();
    //let link_re = Regex::new(r"\[(.+)\]\(.+\)").unwrap();

    let mut things: Vec<Thing> = Vec::new();
    let mut th = Thing{
        header: "".to_string(),
        color: [0.0; 3],
        children: Vec::new(),
    };

    let mut old = "".to_string();
    let lines = contents.split('\n');

    let mut idx = 0;
    for line in lines {
        let mut child_text: Vec<Text> = Vec::new();

        let words = line.split(' ');
        for mut w in words {
            if w.starts_with('@') {
                if w.to_string() != old {
                    if idx > 0 {
                        things.push(th);
                    }

                    th = Thing{
                        header: (&w[1..]).to_string(),
                        color: [0.0; 3],
                        children: Vec::new(),
                    };

                    old = w.to_string();
                }

                if w == "@fix" {
                    th.color = [125.0 / 255.0, 223.0 / 255.0, 100.0 / 255.0];
                } else if w == "@new" {
                    th.color = [125.0 / 255.0, 100.0 / 255.0, 223.0 / 255.0];
                } else if w == "@bug" {
                    th.color = [223.0 / 255.0, 100.0 / 255.0, 125.0 / 255.0];
                }
            } else {
                let mut typ = "regular".to_string();
                let mut modifier = "normal".to_string();

                let bold_match: Vec<_> = bold_re.captures_iter(w).collect();
                if bold_match.len() > 0 {
                    typ = "bold".to_string();
                    w = bold_match[0].get(2).unwrap().as_str();
                }

                let italic_match: Vec<_> = italic_re.captures_iter(w).collect();
                if italic_match.len() > 0 {
                    modifier = "italic".to_string();
                    w = italic_match[0].get(2).unwrap().as_str();
                }

                let mut word: String = w.to_string();
                word.push_str(" ");

                child_text.push(Text{
                    font: format!("static/fonts/roboto/{}-{}.ttf", typ, modifier).to_string(),
                    word,
                });
            }
        }

        th.children.push(TextLine{children: child_text});
        idx += 1;
    }
    
    things.push(th);
    things
}

fn make_pdf(things: Vec<Thing>) {
    let mut height: f64 = 7.0;
    for th in things.iter() {
        for _ in th.children.iter() {
            height += 9.5;
        }

        height += 5.5;
    }

    let (doc, page, layer) = PdfDocument::new("log-print", Mm(210.0), Mm(height), "");
    let curr_layer = doc.get_page(page).get_layer(layer);

    curr_layer.set_line_height(12);
    curr_layer.set_character_spacing(1);

    let l = Line{
        points: vec![
            (Point::new(Mm(0.0), Mm(height-10.0)), false),
            (Point::new(Mm(0.0), Mm(height)), false),
            (Point::new(Mm(210.0), Mm(height)), false),
            (Point::new(Mm(210.0), Mm(height-10.0)), false),
        ],
        is_closed: true,
        has_fill: true,
        has_stroke: false,
        is_clipping_path: false,
    }; 

    curr_layer.set_fill_color(Color::Rgb(Rgb::new(0.8, 0.8, 0.8, None)));
    curr_layer.add_shape(l);

    let font = doc.add_external_font(fs::File::open("static/fonts/roboto/regular-normal.ttf").unwrap()).unwrap();
    curr_layer.set_font(&font, 22);
    curr_layer.set_fill_color(Color::Rgb(Rgb::new(255.0, 255.0, 255.0, None)));

    curr_layer.use_text(&"begin".to_string(), 22, Mm(5.0), Mm(height-7.5), &font);

    let l = Line{
        points: vec![
            (Point::new(Mm(208.0), Mm(height)), false),
            (Point::new(Mm(208.0), Mm(0.0)), false),
            (Point::new(Mm(210.0), Mm(0.0)), false),
            (Point::new(Mm(210.0), Mm(height)), false),
        ],
        is_closed: true,
        has_fill: true,
        has_stroke: false,
        is_clipping_path: false,
    }; 

    curr_layer.set_fill_color(Color::Rgb(Rgb::new(0.8, 0.8, 0.8, None)));
    curr_layer.add_shape(l);

    let mut add_y: f64 = 10.0;
    for th in things.iter() {
        let l = Line{
            points: vec![
                (Point::new(Mm(0.0), Mm(height-10.0-add_y)), false),
                (Point::new(Mm(0.0), Mm(height-add_y)), false),
                (Point::new(Mm(105.0), Mm(height-add_y)), false),
                (Point::new(Mm(105.0), Mm(height-10.0-add_y)), false),
            ],
            is_closed: true,
            has_fill: true,
            has_stroke: false,
            is_clipping_path: false,
        }; 

        curr_layer.set_fill_color(Color::Rgb(Rgb::new(th.color[0], th.color[1], th.color[2], None)));
        curr_layer.add_shape(l);

        let font = doc.add_external_font(fs::File::open("static/fonts/roboto/regular-normal.ttf").unwrap()).unwrap();
        curr_layer.set_font(&font, 22);
        curr_layer.set_fill_color(Color::Rgb(Rgb::new(255.0, 255.0, 255.0, None)));

        curr_layer.use_text(&th.header, 22, Mm(5.0), Mm(height-7.5-add_y), &font);

        let mut sub_y: f64 = height - 9.0;
        for ln in th.children.iter() {
            curr_layer.begin_text_section();
            curr_layer.set_text_cursor(Mm(2.5), Mm(sub_y-add_y));

            for tx in ln.children.iter() {
                let font = doc.add_external_font(fs::File::open(&tx.font).unwrap()).unwrap();
                curr_layer.set_fill_color(Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None)));
                curr_layer.set_font(&font, 12);
                curr_layer.write_text(&tx.word, &font);
            }

            curr_layer.add_line_break();
            curr_layer.end_text_section();

            sub_y -= 9.5;
        }

        sub_y -= 5.5;
        let l = Line{
            points: vec![
                (Point::new(Mm(0.0), Mm(height-10.0-add_y)), false),
                (Point::new(Mm(0.0), Mm(sub_y+10.0-add_y)), false),
                (Point::new(Mm(2.0), Mm(sub_y+10.0-add_y)), false),
                (Point::new(Mm(2.0), Mm(height-10.0-add_y)), false),
            ],
            is_closed: true,
            has_fill: true,
            has_stroke: false,
            is_clipping_path: false,
        }; 

        curr_layer.set_fill_color(Color::Rgb(Rgb::new(th.color[0], th.color[1], th.color[2], None)));
        curr_layer.add_shape(l);
        add_y += height - 10.0 - sub_y;
    }

    let l = Line{
        points: vec![
            (Point::new(Mm(0.0), Mm(height-10.0-add_y+9.5)), false),
            (Point::new(Mm(0.0), Mm(height-add_y+9.5)), false),
            (Point::new(Mm(210.0), Mm(height-add_y+9.5)), false),
            (Point::new(Mm(210.0), Mm(height-10.0-add_y+9.5)), false),
        ],
        is_closed: true,
        has_fill: true,
        has_stroke: false,
        is_clipping_path: false,
    }; 

    curr_layer.set_fill_color(Color::Rgb(Rgb::new(0.6, 0.6, 0.6, None)));
    curr_layer.add_shape(l);

    let font = doc.add_external_font(fs::File::open("static/fonts/roboto/regular-normal.ttf").unwrap()).unwrap();
    curr_layer.set_font(&font, 22);
    curr_layer.set_fill_color(Color::Rgb(Rgb::new(255.0, 255.0, 255.0, None)));

    curr_layer.use_text(&"end".to_string(), 22, Mm(5.0), Mm(height-7.5-add_y+9.5), &font);

    doc.save(&mut BufWriter::new(fs::File::create("log-print.pdf").unwrap())).unwrap();
}

fn main() {
    let mut filename: String = "static/logs/test.md".to_string();

    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        filename = String::from(&args[1]);
    }

    let logs = parse_logs(filename);
    make_pdf(logs);
}
