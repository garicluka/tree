use crate::types::Result;
use crossterm::{
    cursor,
    style::{Color, PrintStyledContent, Stylize},
    QueueableCommand,
};
use std::io::{Stdout, Write};

#[derive(Clone, Copy, Debug, PartialEq)]
struct VirtCell {
    fg: Color,
    bg: Color,
    content: char,
}

impl Default for VirtCell {
    fn default() -> Self {
        Self {
            fg: Color::Reset,
            bg: Color::Reset,
            content: ' ',
        }
    }
}

pub struct VirtTerminal {
    old_cells: Vec<Vec<VirtCell>>,
    new_cells: Vec<Vec<VirtCell>>,
    pub width: usize,
    pub height: usize,
}

impl VirtTerminal {
    pub fn new(width: usize, height: usize) -> Self {
        let old_cells = vec![vec![VirtCell::default(); width]; height];
        let new_cells = vec![vec![VirtCell::default(); width]; height];
        Self {
            old_cells,
            new_cells,
            width,
            height,
        }
    }

    pub fn change_cell(&mut self, row: usize, col: usize, content: char, fg: Color, bg: Color) {
        self.new_cells[row][col].content = content;
        self.new_cells[row][col].fg = fg;
        self.new_cells[row][col].bg = bg;
    }

    pub fn flush(&mut self, stdout: &mut Stdout) -> Result {
        for i in 0..self.height {
            for j in 0..self.width {
                if self.old_cells[i][j] != self.new_cells[i][j] {
                    stdout
                        .queue(cursor::MoveTo(j as u16, i as u16))?
                        .queue(PrintStyledContent(
                            self.new_cells[i][j]
                                .content
                                .with(self.new_cells[i][j].fg)
                                .on(self.new_cells[i][j].bg),
                        ))?;
                }
            }
        }
        stdout.flush()?;
        self.old_cells = self.new_cells.clone();
        self.new_cells = vec![vec![VirtCell::default(); self.width]; self.height];
        Ok(())
    }

    pub fn resize(&mut self, width: usize, height: usize, stdout: &mut Stdout) -> Result {
        self.old_cells = vec![vec![VirtCell::default(); width]; height];
        self.new_cells = vec![vec![VirtCell::default(); width]; height];
        self.width = width;
        self.height = height;

        for i in 0..self.height {
            for j in 0..self.width {
                stdout
                    .queue(cursor::MoveTo(j as u16, i as u16))?
                    .queue(PrintStyledContent(' '.with(Color::Reset).on(Color::Reset)))?;
            }
        }
        Ok(())
    }
}
