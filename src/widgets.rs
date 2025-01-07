use std::io::{Stdout, Write};

use termsize::Size;
use textwrap::wrap as wrap_text;
use crossterm::{
    cursor::{MoveDown, MoveToColumn, MoveUp}, execute, style::{Color, Print, SetForegroundColor}
};

pub struct BoxedText {
    size: Size,
    text: String,
}

impl BoxedText {
    pub fn new(width: u16, title: &str, inner_text: &str) -> Self {
        // TODO: Implement a title for BoxedText
        let wrapped_text = wrap_text(inner_text, width as usize - 2).join("\n");
        let height = 3 + wrapped_text.chars().filter(|c| c == &'\n').count() as u16;

        let size = Size { cols: width, rows: height };
        let text = wrapped_text.to_owned();

        Self {
            size,
            text,
        }
    }

    pub fn draw<T: Write>(&self, out: &mut T) -> anyhow::Result<()> {
        let width = self.size.cols;
        
        // Print top
        // TODO: Add colors to indicate overdue tasks and tasks due today. Make sure they're ordered.
        execute!(
            out,
            Print("┌"),
            Print("─".repeat(width as usize - 3)),
            Print("┐\n"),
            //MoveToColumn(0),
            //MoveDown(2),
            Print(&self.text),
            //MoveDown(1)
            Print("└"),
            Print("─".repeat(width as usize - 3)),
            Print("┘"),
        )?;

        Ok(())
    }
}

