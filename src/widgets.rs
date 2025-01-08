use std::io::Write;

use chrono::{DateTime, Utc};
use google_tasks1::api::Task;
use hyphenation_commons::{dictionary::Builder, Language};
use textwrap::{wrap as wrap_text, Options, WordSplitter};
use crossterm::{
    execute, style::{Color, Print, SetForegroundColor}
};

pub struct Boxed<'a> {
    width: u16,
    tasks: &'a Vec<Task>,
}

impl<'a> Boxed<'a> {
    pub fn new(width: u16, title: &str, tasks: &'a Vec<Task>) -> Self {
        // TODO: Implement a title for BoxedText

        Self {
            width,
            tasks,
        }
    }

    pub fn draw<T: Write>(&self, out: &mut T) -> anyhow::Result<()> {
        let width = self.width;
        
        // Print top of box
        execute!(
            out,
            Print("┌"),
            Print("─".repeat(width as usize - 3)),
            Print("┐\n"),
        )?;

        for task in self.tasks {
            // Split title into lines and add indentation characters
            let title = wrap_text(
                task.title.as_deref().unwrap_or(""),
                Options::new(self.width as usize - 6).subsequent_indent("   ").initial_indent("• ")
                .word_splitter(WordSplitter::Hyphenation(Builder {
                        language: Language::EnglishUS, // TODO: Make this configurable via the TOML
                        patterns: Default::default(),
                        exceptions: Default::default()
                }.into()))
            );

            // Split notes into lines and add indentation characters
            let notes = task.notes.as_ref().map(|notes_str| wrap_text(
                notes_str,
                Options::new(self.width as usize - 6).subsequent_indent("     ").initial_indent("  • ")
                .word_splitter(WordSplitter::Hyphenation(Builder {
                        language: Language::EnglishUS, // TODO: Make this configurable via the TOML
                        patterns: Default::default(),
                        exceptions: Default::default()
                }.into()))
            ).to_vec());

            // Set color based on due date: orange for today, white for future, red for past due
            let color =
                if let Some(ref due_rfc3339) = task.due {
                    match DateTime::parse_from_rfc3339(due_rfc3339) {
                        Err(e) => anyhow::bail!(e),
                        Ok(dt_utc_fixed) => {
                            // TODO: Add support for times, too. Currently, I just look at the days (`chrono::NaiveTime`s)
                            let now = Utc::now();
                            let dt = dt_utc_fixed.date_naive();
                            let now = now.date_naive();

                            match (dt, now) {
                                (dt, now) if dt == now => Color::AnsiValue(208),   // Due today: orange
                                (dt, now) if dt > now => Color::White,             // Due later
                                (dt, now) if dt < now => Color::Red,               // Past due!
                                _ => unsafe { std::hint::unreachable_unchecked() }
                            }
                        }
                    }
                } else {
                    // No due date => white color
                    Color::White
                };

            // Handle title printing
            for line in &title {
                execute!(
                    out,
                    Print('│'),
                    SetForegroundColor(color),
                    Print(line),
                    Print(" ".repeat(self.width as usize - 3 - line.chars().count())),
                    SetForegroundColor(Color::White),
                    Print("│\n"),
                )?;
            }

            // Handle notes printing (if there are notes)
            if let Some(notes) = notes {
                for line in &notes {
                    execute!(out,
                        Print('│'),
                        SetForegroundColor(Color::Grey),
                        Print(line),
                        Print(" ".repeat(self.width as usize - 3 - line.chars().count()) + "│\n"),
                    )?;
                }
            }
        }

        // Print bottom of box
        execute!(
            out,
            Print("└"),
            Print("─".repeat(width as usize - 3)),
            Print("┘"),
        )?;

        Ok(())
    }
}

