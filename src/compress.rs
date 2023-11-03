//! Makes sure as little data is sent to GPU as possible so that this is faster

use egui::Rect;
use micro_ndarray::Array;

pub struct ChangedRect {
    pub changelist: Option<Vec<[usize; 2]>>,
    pub count: usize,
    pub rect: Rect,
    pub min: [usize; 2],
    pub max: [usize; 2],
    pub size: [usize; 2],
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
            self.changelist = None;
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
        self.push(rect.min.x as usize, rect.min.y as usize);
        self.push(rect.max.x as usize, rect.max.y as usize);
        self.changelist = None;
        self.count += rect.area() as usize - 2;
    }

    pub fn take(&mut self) -> ChangedRect {
        self.empty = true;
        let count = self.count;
        let changelist = self.changelist.take();
        self.count = 0;
        self.changelist = Some(Vec::with_capacity(self.max_changelist_len));
        ChangedRect {
            changelist,
            count,
            rect: Rect {
                min: self.min.map(|x| x as f32).into(),
                max: self.max.map(|x| x as f32).into(),
            },
            min: self.min,
            max: self.max,
            size: [self.max[0] - self.min[0], self.max[1] - self.min[1]],
        }
    }
}

pub trait FlatArea<T> {
    fn area_flat(&self, area: Rect) -> Vec<T>;
}

impl<T: Copy + Sized> FlatArea<T> for Array<T, 2> {
    fn area_flat(&self, area: Rect) -> Vec<T> {
        let size = area.size();
        let size = [size.x as usize, size.y as usize];
        let mut r = Vec::with_capacity(area.area() as usize);
        let y_len = self.size()[0];
        let self_flat = self.as_flattened();
        // SAFETY: [all unsafe operations explained in further comments]
        unsafe {
            let mut r_ptr = r.as_mut_ptr();
            // SAFETY: Every element will be overwritten
            r.set_len(size[0] * size[1]);
            for i in 0..size[1] {
                let idx = area.min.x as usize + (area.min.y as usize + i) * y_len;
                // SAFETY: All items are [`Copy`], both pointers have the same
                // types and cannot overlap as r is freshly created. The length
                // of r is ensured above at r.set_len, based on the length of
                // self_flat.
                self_flat[idx..]
                    .as_ptr()
                    .copy_to_nonoverlapping(r_ptr, size[0]);
                r_ptr = r_ptr.offset(size[0] as isize);
            }
        }
        r
    }
}
