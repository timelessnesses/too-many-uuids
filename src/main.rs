use crossterm::event;
use ratatui::{layout::{Constraint, Layout, Rect}, style::{Color, Style}, text::Text, widgets::{Block, Borders, Clear, List, Paragraph}};
use std::time::Duration;
mod attempt;

fn main() {
    let mut terminal = ratatui::init();
    terminal.autoresize().unwrap();
    let mut start_index = 0u128;
    let mut finding_window_opened = false;
    let mut flashing_index = 0;
    let mut flash_now= false;
    let mut text_input = tui_textarea::TextArea::default();
    text_input.set_placeholder_text("Enter a UUID to find");
    text_input.set_placeholder_style(Style::default().fg(Color::LightYellow));
    text_input.set_cursor_line_style(Style::default());
    text_input.set_block(Block::default().borders(Borders::ALL).title("Find UUID"));
    loop {
        terminal.draw(|i| {
            let chunks = Layout::default().direction(ratatui::layout::Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Percentage(80), Constraint::Max(4)])
                .split(i.area());
            
            let items = (start_index..start_index + i.area().width as u128).map(|i| {
                let index = format!("{:0>32}", i);
                let uuid = format!("{}", index_to_uuid(i));
                let padding_space = index.len() + uuid.len();
                let width = chunks[0].width as usize;
                let padding = if padding_space < width {
                    " ".repeat(width - padding_space - 4)
                } else {
                    "".to_string()
                };
                if flash_now && i == flashing_index {
                    Text::styled(format!(" {}{}{}", index, padding, uuid), Style::default().fg(Color::Cyan))
                } else {
                    Text::styled(format!(" {}{}{}", index, padding, uuid), Style::default().fg(Color::White))
                }
            });
            let list = List::new(items).block(Block::default().borders(Borders::ALL).title("UUIDs"))
                .highlight_symbol(">> ");
            i.render_widget(list, chunks[0]);
            let text = Paragraph::new("Press q to quit, Press PGUP/PGDOWN to scroll, G to open a find window to get to specific index").block(Block::default().borders(Borders::ALL).title("Instructions"));
            i.render_widget(text, chunks[1]);
            if finding_window_opened {
                let area = popup_area(i.area(), 50, 50);
                let block = Block::default().borders(Borders::ALL).title("Finding UUID");
                let text = Paragraph::new("Press ESC to quit, Press Enter to go to that index").block(block).centered();
                i.render_widget(Clear, area);
                i.render_widget(text, area);
                let area = popup_area(area, 50, 50);
                i.render_widget(&text_input, area);

            }
        }).unwrap();
        if event::poll(Duration::from_millis(5)).unwrap() {
            if let event::Event::Key(key) = event::read().unwrap() {
                match key.code {
                    event::KeyCode::PageDown  | event::KeyCode::Down => {
                        start_index += 1;
                        if start_index >= 2u128.pow(122) {
                            start_index = 0;
                        }
                        flash_now = false;
                    },
                    event::KeyCode::PageUp | event::KeyCode::Up => {
                        start_index -= 1;
                        if start_index >= 2u128.pow(122) {
                            start_index = 0;
                        }
                    },
                    event::KeyCode::Char('q') => {
                        if !finding_window_opened {
                            break;
                        }
                    },
                    event::KeyCode::Char('g') => {
                        if !finding_window_opened {
                            finding_window_opened = true;
                        }
                    },
                    event::KeyCode::Esc => {
                        if finding_window_opened {
                            finding_window_opened = false;
                        }
                    },
                    event::KeyCode::Enter => {
                        if finding_window_opened {
                            let index = text_input.lines()[0].trim().parse::<u128>().expect("invalid index");
                            text_input.delete_line_by_head();
                            finding_window_opened = false;
                            start_index = index;
                            flashing_index = index;
                            flash_now = true;
                        }
                    }
                    _ => {
                        if finding_window_opened {
                            text_input.input(key);
                            validate_index(text_input.lines()[0].parse::<u128>(), &mut text_input);
                        }
                    }
                }
            }
        }
    }
    terminal.clear().unwrap()
}

fn validate_index(index: Result<u128, std::num::ParseIntError>, text_input: &mut tui_textarea::TextArea) {
    let index = match index {
        Ok(index) => index,
        Err(_) => {
            text_input.set_style(Style::default().bg(Color::Red));
            text_input.set_block(Block::default().borders(Borders::ALL).title("Invalid index"));
            return
        }
    };
    if index >= 2u128.pow(122) {
        text_input.set_style(Style::default().bg(Color::Red));
        text_input.set_block(Block::default().borders(Borders::ALL).title("Index out of bounds"));
    } else {
        text_input.set_style(Style::default().bg(Color::Green));
        text_input.set_block(Block::default().borders(Borders::ALL).title("Find UUID"));
    }
}

const ROUNDS: usize = 4;
const ROUND_CONSTANTS: [u128; 8] = [
    0x47f5417d6b82b5d1,
    0x90a7c5fe8c345af2,
    0xd8796c3b2a1e4f8d,
    0x6f4a3c8e7d5b9102,
    0xb3f8c7d6e5a49201,
    0x2d9e8b7c6f5a3d4e,
    0xa1b2c3d4e5f6789a,
    0x123456789abcdef0,
];

/// converts index (0 - (2^122 - 1)) to uuid
/// implementation from https://github.com/nolenroyalty/every-uuid/blob/main/lib/uuidTools.js#L57
fn index_to_uuid(index: u128) -> String {
    if index >= 2u128.pow(122) {
        panic!("index out of bounds");
    }
    let mut left = index >> 61;
    let mut right = index & ((1 << 61) - 1);
    for x in 0..ROUNDS {
        let calculated = feistel(right, x);
        let new_right = left ^ (calculated & ((1 << 61) - 1));
        left = right;
        right = new_right;
    }
    let mut result = 0;
    result |= (left >> 13) << 80;
    result |= 4 << 76;
    let next_12_bits_from_left = (left >> 1) & ((1 << 12) - 1);
    result |= next_12_bits_from_left << 64;
    result |= 2 << 62;
    let last_bit_from_left = left & 1;
    result |= last_bit_from_left << 61;
    result |= right;
    let hex = format!("{:X}", result);
    let hex = format!("{:0>32}", hex);
    format!("{}-{}-{}-{}-{}", &hex[0..8], &hex[8..12], &hex[12..16], &hex[16..20], &hex[20..]).to_lowercase()
}

fn feistel(block: u128, rounds: usize) -> u128 {
    let mut result = block;
    result ^= ROUND_CONSTANTS[rounds] & ((1 << 61) - 1);
    result = result << 7 | result >> 54 & ((1 << 61) - 1);
    result = result * 0x6c8e944d1f5aa3b7 & ((1 << 61) - 1);
    result = result << 13 | result >> 48 & ((1 << 61) - 1);
    result
}

fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(ratatui::layout::Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(ratatui::layout::Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}