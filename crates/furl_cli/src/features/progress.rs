use crate::ProgressReporter;

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::sync::Mutex;

pub struct GraphicalProgressReporter {
    multi: MultiProgress,
    bars: Mutex<Vec<ProgressBar>>,
}

impl GraphicalProgressReporter {
    pub fn new() -> Self {
        Self {
            multi: MultiProgress::new(),
            bars: Mutex::new(Vec::new()),
        }
    }

    fn get_or_create_bar(&self, chunk_index: usize, total_bytes: u64) -> ProgressBar {
        let mut bars = self.bars.lock().unwrap();

        // grow the vec if needed
        while bars.len() <= chunk_index {
            let bar = self.multi.add(ProgressBar::new(0));
            bars.push(bar);
        }

        let bar = &bars[chunk_index];
        if total_bytes > 0 {
            // known size: labeled progress bar
            bar.set_length(total_bytes);
            bar.set_style(
                ProgressStyle::with_template(&format!(
                    "[Chunk {:03}] {{wide_bar:40.cyan/blue}} {{binary_bytes}}/{{binary_total_bytes}} ({{percent}}%)",
                    chunk_index + 1
                ))
                .unwrap()
                // .progress_chars("=>-"),
            );
        } else {
            // unknown size: spinner
            bar.set_style(
                ProgressStyle::with_template(
                    "{spinner:.cyan} {wide_msg} ({binary_bytes} downloaded)",
                )
                .unwrap()
                .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"),
            );
            bar.enable_steady_tick(std::time::Duration::from_millis(100));
        }
        bar.clone()
    }
}

impl Default for GraphicalProgressReporter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "progress")]
impl ProgressReporter for GraphicalProgressReporter {
    fn on_start(&self, chunk_index: usize, total_bytes: u64) {
        self.get_or_create_bar(chunk_index, total_bytes);
    }

    fn on_progress(&self, chunk_index: usize, bytes_downloaded: u64) {
        let bars = self.bars.lock().unwrap();
        if let Some(bar) = bars.get(chunk_index) {
            bar.set_position(bytes_downloaded);
        }
    }

    fn on_finish(&self, chunk_index: usize) {
        let bars = self.bars.lock().unwrap();
        if let Some(bar) = bars.get(chunk_index) {
            bar.finish();
        }
    }
}
