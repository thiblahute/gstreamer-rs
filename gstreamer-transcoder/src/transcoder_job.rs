// Copyright (C) 2018 Thibault Saunier <tsaunier@igalia.com>
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use std::cell::RefCell;

#[repr(C)]
struct Source {
    uri: String,    
}

#[repr(C)]
struct TranscoderJob {
    // FIXME What would be a better way to do that?
    sources: RefCell<Vec<Source>>,
}

impl TranscoderJob {
    pub fn new() -> Self {
        Self {
            sources: RefCell::new(Vec::new())
        }

    }
}