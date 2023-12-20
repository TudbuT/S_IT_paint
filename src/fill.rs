use std::collections::HashSet;

use egui::Color32;

use crate::{
    draw::{DrawParams, Location},
    App,
};

/// The state struct for the fill algorithm
struct Filler<'app> {
    col: Color32,
    open: Vec<Location>,
    closed: HashSet<Location>,
    fill_color: Color32,
    app: &'app mut App,
}

impl<'app> Filler<'app> {
    fn new(app: &'app mut App, draw: DrawParams) -> Self {
        Self {
            col: app.image[[draw.loc.x, draw.loc.y]], // color to replace
            open: vec![draw.loc],
            closed: HashSet::with_capacity(128),
            // color to fill with
            fill_color: Color32::from_rgb(
                (draw.px >> 16) as u8,
                (draw.px >> 8) as u8,
                draw.px as u8,
            ),
            app,
        }
    }

    fn push(&mut self, node: Location) {
        // has this been visited yet?
        if !self.closed.contains(&node) {
            self.open.push(node);
        }
    }

    /// Fills the area.
    ///
    /// "nodes" are pixels
    /// 1. add current pixel to open list
    /// 2. pop first pixel from open list
    /// 3. add it to closed list
    /// 4. check if it is valid
    /// 5. color it
    /// 6. add its neighbors to the open list
    /// 7. repeat from 2 on until open list is empty
    fn fill(&mut self) {
        let img_size = self.app.image.size();
        // until all nodes are visited
        while let Some(node) = self.open.pop() {
            // mark node done
            self.closed.insert(node);
            // if node is invalid, skip it
            if node.x >= img_size[0] || node.y >= img_size[1] {
                continue;
            }
            // if node is wrong color, skip it
            if self.app.image[[node.x, node.y]] != self.col {
                continue;
            }

            // set color without bounds check (already done above)
            self.app.set_px_unchecked(node.x, node.y, self.fill_color);

            // mark neighbors to be visited
            self.push(node.offset(-1, 0));
            self.push(node.offset(1, 0));
            self.push(node.offset(0, -1));
            self.push(node.offset(0, 1));
        }
    }
}

impl App {
    pub fn fill(&mut self, draw: DrawParams) {
        Filler::new(self, draw).fill();
    }
}
