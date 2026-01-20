use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum AnnotationType {
    Text = 1,
    Line = 2,
    Geom = 3,
    Highlight = 4,
    Stamp = 5,
    Ink = 6,
    Caret = 8,
    FileAttachment = 9,
    Sound = 10,
    Movie = 11,
    Screen = 12,
    Widget = 13,
    RichMedia = 14,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NormalizedRect {
    pub left: f64,
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Annotation {
    pub id: String,
    pub annotation_type: AnnotationType,
    pub author: String,
    pub contents: String,
    pub unique_name: String,
    pub creation_date: String,     // ISO 8601
    pub modification_date: String, // ISO 8601
    pub flags: i32,
    pub bounding_rect: NormalizedRect,
    // Style properties can be expanded here
    pub color: String, // Hex code
    pub opacity: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Page {
    pub width: f64,
    pub height: f64,
    pub annotations: Vec<Annotation>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Document {
    pub pages: HashMap<u32, Page>, // Page number -> Page
    pub metadata: HashMap<String, String>,
}

impl Document {
    pub fn new() -> Self {
        Document {
            pages: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn add_annotation(&mut self, page_num: u32, annotation: Annotation) {
        let page = self.pages.entry(page_num).or_insert(Page {
            width: 0.0,
            height: 0.0,
            annotations: Vec::new(),
        });
        page.annotations.push(annotation);
    }
}
