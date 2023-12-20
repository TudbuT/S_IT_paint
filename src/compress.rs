//! Makes sure as little data is sent to GPU as possible so that this is faster

use egui::Rect;
use micro_ndarray::Array;

pub struct ChangedRect {
    pub changelist: Option<Vec<[usize; 2]>>,
    pub count: usize,
    pub min: [usize; 2],
    pub max: [usize; 2],
    pub size: [usize; 2],
    pub area: usize,
}

pub struct ChangeRect {
    empty: bool,
    max_changelist_len: usize,
    changelist: Option<Vec<[usize; 2]>>,
    min: [usize; 2],
    max: [usize; 2],
    count: usize,
}

impl ChangeRect {
    pub fn new(max_changelist_len: usize) -> Self {
        Self {
            empty: true,
            max_changelist_len,
            changelist: Some(Vec::with_capacity(max_changelist_len)),
            min: [0; 2],
            max: [0; 2],
            count: 0,
        }
    }

    pub fn push(&mut self, x: usize, y: usize) {
        self.count += 1;
        if self.changelist.is_some() && self.count >= self.max_changelist_len {
            self.changelist = None; // changelist has "overflown" (too much to update single points)
        }
        if let Some(ref mut changelist) = self.changelist {
            changelist.push([x, y]);
        }
        if self.empty {
            self.min = [x, y];
            self.max = [x, y];
            self.empty = false;
            return;
        }

        // expand area to include this point
        if x < self.min[0] {
            self.min[0] = x;
        }
        if x > self.max[0] {
            self.max[0] = x;
        }
        if y < self.min[1] {
            self.min[1] = y;
        }
        if y > self.max[1] {
            self.max[1] = y;
        }
    }

    pub fn all(&mut self, rect: Rect) {
        // only pushes the corners as an optimization
        self.push(rect.min.x as usize, rect.min.y as usize);
        self.push(rect.max.x as usize, rect.max.y as usize);
        self.changelist = None; // force "overflown" changelist
        self.count += rect.area() as usize - 2; // add other pixels in rectangle that werent added by push
    }

    /// "takes" the changes, resetting this struct and returning the area/pixels to update
    pub fn take(&mut self) -> ChangedRect {
        self.empty = true;
        let count = self.count;
        let changelist = self.changelist.take();
        self.count = 0;
        self.changelist = Some(Vec::with_capacity(self.max_changelist_len));

        let size = [self.max[0] - self.min[0] + 1, self.max[1] - self.min[1] + 1];
        ChangedRect {
            changelist,
            count,
            min: self.min,
            max: self.max,
            size,
            area: size[0] * size[1],
        }
    }
}

pub trait FlatArea<T> {
    fn area_flat(&self, start: [usize; 2], size: [usize; 2]) -> Vec<T>;
}

impl<T: Copy + Sized> FlatArea<T> for Array<T, 2> {
    fn area_flat(&self, start: [usize; 2], size: [usize; 2]) -> Vec<T> {
        let array_size = self.size();
        if start[0] + size[0] > array_size[0] || start[1] + size[1] > array_size[1] {
            panic!("Out of bounds area_flat: Size {size:?} at {start:?} requested, but only {array_size:?} available.")
        }
        let mut r = Vec::with_capacity(size[0] * size[1]);
        let y_len = self.size()[0];
        let self_flat = self.as_flattened();
        // SAFETY: [all unsafe operations explained in further comments]
        unsafe {
            let mut r_ptr = r.as_mut_ptr();
            // SAFETY: Every element will be overwritten => no garbage data will be left
            r.set_len(size[0] * size[1]);
            for i in 0..size[1] {
                let idx = start[0] + (start[1] + i) * y_len;
                // SAFETY: All items are [`Copy`], both pointers have the same
                // types and cannot overlap as r is freshly created. The length
                // of r is ensured above at r.set_len, based on the length of
                // self_flat.
                self_flat[idx..]
                    .as_ptr()
                    .copy_to_nonoverlapping(r_ptr, size[0]);
                r_ptr = r_ptr.add(size[0]);
            }
        }
        r
    }
}
