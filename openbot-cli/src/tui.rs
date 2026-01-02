use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    DefaultTerminal,
};
use std::sync::mpsc::Receiver;
use std::time::Duration;
use ansi_to_tui::IntoText;
use crate::config; // 确保你的 main.rs 导出了 config 宏

pub struct TuiApp {
    input: String,
    logs: Vec<String>,
    scroll_offset: usize,
}

impl TuiApp {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            logs: Vec::new(),
            scroll_offset: 0,
        }
    }

    pub fn run(mut self, mut terminal: DefaultTerminal, log_rx: Receiver<String>) -> anyhow::Result<()> {
        loop {
            // --- 1. 绘制界面 ---
            terminal.draw(|f| {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Min(0), Constraint::Length(3)])
                    .split(f.area());

                let visible_height = chunks[0].height.saturating_sub(2) as usize;
                let total_logs = self.logs.len();

                // 滚动界限计算
                let max_scroll = total_logs.saturating_sub(visible_height);
                self.scroll_offset = self.scroll_offset.min(max_scroll);

                let end = total_logs.saturating_sub(self.scroll_offset);
                let start = end.saturating_sub(visible_height);

                // 渲染日志：支持 RGB ANSI 颜色解析
                let display_logs: Vec<ListItem> = self.logs[start..end]
                    .iter()
                    .map(|s| {
                        match s.as_bytes().into_text() {
                            Ok(text) => ListItem::new(text),
                            Err(_) => ListItem::new(s.as_str()),
                        }
                    })
                    .collect();

                let log_title = format!(" Logs [Showing: {}-{} / Total: {}] ", start + 1, end, total_logs);

                f.render_widget(
                    List::new(display_logs).block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(log_title)
                            .border_style(Style::default().fg(Color::Rgb(100, 100, 100)))
                    ),
                    chunks[0],
                );

                // 渲染输入框
                f.render_widget(
                    Paragraph::new(format!(">> {}", self.input))
                        .style(Style::default().fg(Color::Rgb(0, 255, 255)))
                        .block(Block::default()
                            .borders(Borders::ALL)
                            .title(" Input ")
                            .border_style(Style::default().fg(Color::Rgb(0, 150, 255)))
                        ),
                    chunks[1],
                );

                // 设置光标
                f.set_cursor_position((
                    chunks[1].x + 4 + self.input.chars().count() as u16,
                    chunks[1].y + 1,
                ));
            })?;

            // --- 2. 事件处理 (阻塞式以降低 CPU) ---
            // 建议将 refresh_millis 设置为 10-50 之间，即使是 0，通过 poll 也会大幅降低空转负载
            let timeout = Duration::from_millis(config!().tui.refresh_millis.max(5));
            if event::poll(timeout)? {
                match event::read()? {
                    Event::Key(key) => {
                        if key.kind == KeyEventKind::Release {
                            continue;
                        }
                        match key.code {
                            KeyCode::Char(c) => self.input.push(c),
                            KeyCode::Backspace => { self.input.pop(); },
                            KeyCode::PageUp => {
                                let max_scroll = self.logs.len().saturating_sub(3);
                                self.scroll_offset = (self.scroll_offset + 5).min(max_scroll);
                            }
                            KeyCode::PageDown => {
                                self.scroll_offset = self.scroll_offset.saturating_sub(5);
                            }
                            KeyCode::Enter => {
                                let cmd = self.input.trim().to_string();
                                if !cmd.is_empty() {
                                    if cmd == "exit" { break; }
                                    tracing::warn!("Unknown command: {}", cmd);
                                    self.input.clear();
                                    self.scroll_offset = 0;
                                }
                            }
                            KeyCode::Esc => break,
                            _ => {}
                        }
                    }
                    // 处理缩放，让重绘更及时
                    Event::Resize(_, _) => {}
                    _ => {}
                }
            }

        // --- 3.as
            let mut recieved = false;
            while let Ok(new_log) = log_rx.try_recv()
            {
                self.logs.push(new_log);
                recieved = true;
                
            }
            // 日志阈值
            if self.logs.len() > config!().tui.logs_max{
                self.logs.drain(0..self.logs.len());
            }
            
        }
        Ok(())
    }
        
}