#![allow(dead_code)]
#![feature(core)]
#![feature(collections)]
#![feature(unicode)]
#![feature(test)]

extern crate test;

mod string_utils;
mod tests;
mod benches;

use std::cmp::{min, max};
use std::mem;
use std::str::Graphemes;
use std::ops::Index;
use string_utils::{
    char_count,
    char_grapheme_line_ending_count,
    grapheme_count_is_less_than,
    insert_text_at_grapheme_index,
    remove_text_between_grapheme_indices,
    split_string_at_grapheme_index,
    is_line_ending,
};


pub const MIN_NODE_SIZE: usize = 64;
pub const MAX_NODE_SIZE: usize = MIN_NODE_SIZE * 2;


/// A rope data structure for storing text in a format that is efficient
/// for insertion and removal even for extremely large strings.
#[derive(Debug)]
pub struct Rope {
    data: RopeData,
    char_count_: usize,
    grapheme_count_: usize,
    line_ending_count: usize,
    tree_height: u32,
}


#[derive(Debug)]
enum RopeData {
    Leaf(String),
    Branch(Box<Rope>, Box<Rope>),
}


impl Rope {
    /// Creates a new empty rope
    pub fn new() -> Rope {
        Rope {
            data: RopeData::Leaf(String::new()),
            char_count_: 0,
            grapheme_count_: 0,
            line_ending_count: 0,
            tree_height: 1,
        }
    }
    

    /// Creates a new rope from a string slice    
    pub fn from_str(s: &str) -> Rope {
        let mut rope_stack: Vec<Rope> = Vec::new();
        
        let mut s1 = s;
        loop {
            // Get the next chunk of the string to add
            let mut byte_i = 0;
            let mut le_count = 0;
            let mut c_count = 0;
            let mut g_count = 0;
            for (bi, g) in s1.grapheme_indices(true) {
                byte_i = bi + g.len();
                g_count += 1;
                c_count += char_count(g);
                if is_line_ending(g) {
                    le_count += 1;
                }
                if g_count >= MAX_NODE_SIZE {
                    break;
                }
            }
            if g_count == 0 {
                break;
            }
            let chunk = &s1[..byte_i];
            
            // Add chunk
            rope_stack.push(Rope {
                data: RopeData::Leaf(String::from_str(chunk)),
                char_count_: c_count,
                grapheme_count_: g_count,
                line_ending_count: le_count,
                tree_height: 1,
            });
            
            // Do merges
            loop {
                let rsl = rope_stack.len();
                if rsl > 1 && rope_stack[rsl-2].tree_height <= rope_stack[rsl-1].tree_height {
                    let right = Box::new(rope_stack.pop().unwrap());
                    let left = Box::new(rope_stack.pop().unwrap());
                    let h = max(left.tree_height, right.tree_height) + 1;
                    let lc = left.line_ending_count + right.line_ending_count;
                    let gc = left.grapheme_count_ + right.grapheme_count_;
                    let cc = left.char_count_ + right.char_count_;
                    rope_stack.push(Rope {
                        data: RopeData::Branch(left, right),
                        char_count_: cc,
                        grapheme_count_: gc,
                        line_ending_count: lc,
                        tree_height: h,
                    });
                }
                else {
                    break;
                }
            }
            
            s1 = &s1[byte_i..];
        }
        
        
        // Handle possible final unmerged case
        let rope = if rope_stack.len() == 0 {
            Rope::new()
        }
        else {
            while rope_stack.len() > 1 {
                let right = rope_stack.pop().unwrap();
                let mut left = rope_stack.pop().unwrap();
                left.append_right(right);
                rope_stack.push(left);
            }
            rope_stack.pop().unwrap()
        };
        
        return rope;
    }
    
    pub fn from_str_with_count(s: &str, c_count: usize, g_count: usize, le_count: usize) -> Rope {
        if g_count <= MAX_NODE_SIZE {
            Rope {
                data: RopeData::Leaf(String::from_str(s)),
                char_count_: c_count,
                grapheme_count_: g_count,
                line_ending_count: le_count,
                tree_height: 1,
            }
        }
        else {
            Rope::from_str(s)
        }
    }
    
    /// Creates a new rope from a string, consuming the string
    pub fn from_string(s: String) -> Rope {
        // TODO: special case short strings?
        Rope::from_str(s.as_slice())
    }
    
    pub fn char_count(&self) -> usize {
        return self.char_count_;
    }
    
    pub fn grapheme_count(&self) -> usize {
        return self.grapheme_count_;
    }
    
    pub fn line_count(&self) -> usize {
        return self.line_ending_count + 1;
    }
    
    
    /// Returns the number of graphemes between char indices pos_a and pos_b.
    /// This is not as simple as a subtraction of char_index_to_grapheme_index()
    /// calls, because the char indices may split graphemes.
    pub fn grapheme_count_in_char_range(&self, pos_a: usize, pos_b: usize) -> usize {
        unimplemented!()
    }
    
    
    /// Returns the index of the grapheme that the given char index is a
    /// part of.
    pub fn char_index_to_grapheme_index(&self, pos: usize) -> usize {
        unimplemented!()
    }
    
    
    /// Returns the beginning char index of the given grapheme index.
    pub fn grapheme_index_to_char_index(&self, pos: usize) -> usize {
        unimplemented!()
    }
    
    
    /// Returns the grapheme index at the start of the given line index.
    pub fn line_index_to_grapheme_index(&self, li: usize) -> usize {
        // Bounds check
        if li > self.line_ending_count {
            panic!("Rope::line_index_to_grapheme_index: line index is out of bounds.");
        }
        
        // Special case for the beginning of the rope
        if li == 0 {
            return 0;
        }
        
        // General cases
        match self.data {
            RopeData::Leaf(ref text) => {
                let mut gi = 0;
                let mut lei = 0;
                for g in text.as_slice().graphemes(true) {
                    gi += 1;
                    if is_line_ending(g) {
                        lei += 1;
                    }
                    if lei == li {
                        break;
                    }
                }
                return gi;
            },
            
            RopeData::Branch(ref left, ref right) => {
                if li <= left.line_ending_count {
                    return left.line_index_to_grapheme_index(li);
                }
                else {
                    return right.line_index_to_grapheme_index(li - left.line_ending_count) + left.grapheme_count_;
                }
            },
        }
    }
    
    
    /// Returns the index of the line that the given grapheme index is on.
    pub fn grapheme_index_to_line_index(&self, pos: usize) -> usize {
        match self.data {
            RopeData::Leaf(ref text) => {
                let mut gi = 0;
                let mut lei = 0;
                for g in text.as_slice().graphemes(true) {
                    if gi == pos {
                        break;
                    }
                    gi += 1;
                    if is_line_ending(g) {
                        lei += 1;
                    }
                }
                return lei;
            },
            
            RopeData::Branch(ref left, ref right) => {
                if pos < left.grapheme_count_ {
                    return left.grapheme_index_to_line_index(pos);
                }
                else {
                    return right.grapheme_index_to_line_index(pos - left.grapheme_count_) + left.line_ending_count;
                }
            },
        }
    }
    
    
    /// Converts a grapheme index into a line number and grapheme-column
    /// number.
    ///
    /// If the index is off the end of the text, returns the line and column
    /// number of the last valid text position.
    pub fn grapheme_index_to_line_col(&self, pos: usize) -> (usize, usize) {
        let p = min(pos, self.grapheme_count_);
        let line = self.grapheme_index_to_line_index(p);
        let line_pos = self.line_index_to_grapheme_index(line);
        return (line, p - line_pos);
    }
    
    
    /// Converts a line number and grapheme-column number into a grapheme
    /// index.
    ///
    /// If the column number given is beyond the end of the line, returns the
    /// index of the line's last valid position.  If the line number given is
    /// beyond the end of the buffer, returns the index of the buffer's last
    /// valid position.
    pub fn line_col_to_grapheme_index(&self, pos: (usize, usize)) -> usize {
        if pos.0 <= self.line_ending_count {
            let l_begin_pos = self.line_index_to_grapheme_index(pos.0);
            
            let l_end_pos = if pos.0 < self.line_ending_count {
                self.line_index_to_grapheme_index(pos.0 + 1) - 1
            }
            else {
                self.grapheme_count_
            };
                
            return min(l_begin_pos + pos.1, l_end_pos);
        }
        else {
            return self.grapheme_count_;
        }
    }
    
    
    pub fn char_at_index(&self, index: usize) -> char {
        unimplemented!()
    }
    
    
    pub fn grapheme_at_index<'a>(&'a self, index: usize) -> &'a str {
        &self[index]
    }
    
    
    /// Inserts the given text at the given char index.
    pub fn insert_text_at_char_index(&mut self, text: &str, pos: usize) {
        unimplemented!()
    }
    
    
    /// Removes the text between the given char indices.
    pub fn remove_text_between_char_indices(&mut self, pos_a: usize, pos_b: usize) {
        unimplemented!()
    }
    
    
    /// Inserts the given text at the given grapheme index.
    /// For small lengths of 'text' runs in O(log N) time.
    /// For large lengths of 'text', dunno.  But it seems to perform
    /// sub-linearly, at least.
    pub fn insert_text_at_grapheme_index(&mut self, text: &str, pos: usize) {
        let mut leaf_insert = false;
        
        match self.data {
            // Find node for text to be inserted into
            RopeData::Branch(ref mut left, ref mut right) => {
                if pos < left.grapheme_count_ {
                    left.insert_text_at_grapheme_index(text, pos);
                }
                else {
                    right.insert_text_at_grapheme_index(text, pos - left.grapheme_count_);
                }
            },
            
            // Insert the text
            RopeData::Leaf(ref mut s_text) => {
                if grapheme_count_is_less_than(text, MAX_NODE_SIZE - self.grapheme_count_ + 1) {
                    // Simple case
                    insert_text_at_grapheme_index(s_text, text, pos);
                }
                else {
                    // Special cases
                    leaf_insert = true;
                }
            },
        }
        
        // The special cases of inserting at a leaf node.
        // These have to be done outside of the match statement because
        // of the borrow checker, but logically they take place in the
        // RopeData::Leaf branch of the match statement above.
        if leaf_insert {
            // TODO: these special cases are currently prone to causing leaf
            // fragmentation.  Find ways to reduce that.
            if pos == 0 {
                let mut new_rope = Rope::new();
                mem::swap(self, &mut new_rope);
                self.data = RopeData::Branch(Box::new(Rope::from_str(text)), Box::new(new_rope));
            }
            else if pos == self.grapheme_count_ {
                let mut new_rope = Rope::new();
                mem::swap(self, &mut new_rope);
                self.data = RopeData::Branch(Box::new(new_rope), Box::new(Rope::from_str(text)));
            }
            else {
                // Split the leaf node at the insertion point
                let mut node_l = Rope::new();
                let node_r = self.split_at_grapheme_index(pos);
                mem::swap(self, &mut node_l);
                
                // Set the inserted text as the main node
                *self = Rope::from_str(text);
                
                // Append the left and right split nodes to either side of
                // the main node.
                self.append_left(node_l);
                self.append_right(node_r);
            }
        }
        
        self.update_stats();
        self.rebalance();
    }
    
    
    /// Removes the text between grapheme indices pos_a and pos_b.
    /// For small distances between pos_a and pos_b runs in O(log N) time.
    /// For large distances, dunno.  If it becomes a performance bottleneck,
    /// can special-case that to two splits and an append, which are all
    /// sublinear.
    pub fn remove_text_between_grapheme_indices(&mut self, pos_a: usize, pos_b: usize) {
        // Bounds checks
        if pos_a > pos_b {
            panic!("Rope::remove_text_between_grapheme_indices(): pos_a must be less than or equal to pos_b.");
        }
        if pos_b > self.grapheme_count_ {
            panic!("Rope::remove_text_between_grapheme_indices(): attempt to remove text after end of node text.");
        }
        
        match self.data {
            RopeData::Leaf(ref mut text) => {
                remove_text_between_grapheme_indices(text, pos_a, pos_b);
            },
            
            RopeData::Branch(ref mut left, ref mut right) => {
                let lgc = left.grapheme_count_;
                
                if pos_a < lgc {
                    left.remove_text_between_grapheme_indices(pos_a, min(pos_b, lgc));
                }
                
                if pos_b > lgc {
                    right.remove_text_between_grapheme_indices(pos_a - min(pos_a, lgc), pos_b - lgc);
                }
            }
        }
        
        self.update_stats();
        self.merge_if_too_small();
        self.rebalance();
    }
    
    
    /// Splits a rope into two pieces from the given char index.
    /// The first piece remains in this rope, the second piece is returned
    /// as a new rope.
    /// I _think_ this runs in O(log N) time, but this needs more analysis to
    /// be sure.  It is at least sublinear.
    pub fn split_at_char_index(&mut self, pos: usize) -> Rope {
        unimplemented!()
    }
    
    
    /// Splits a rope into two pieces from the given grapheme index.
    /// The first piece remains in this rope, the second piece is returned
    /// as a new rope.
    /// I _think_ this runs in O(log N) time, but this needs more analysis to
    /// be sure.  It is at least sublinear.
    pub fn split_at_grapheme_index(&mut self, pos: usize) -> Rope {
        let mut left = Rope::new();
        let mut right = Rope::new();
        
        self.split_recursive(pos, &mut left, &mut right);
        
        mem::swap(self, &mut left);
        return right;
    }
    

    /// Appends another rope to the end of this one, consuming the other rope.
    /// Runs in O(log N) time.
    pub fn append(&mut self, rope: Rope) {
        if self.grapheme_count_ == 0 {
            let mut r = rope;
            mem::swap(self, &mut r);
        }
        else if rope.grapheme_count_ == 0 {
            return;
        }
        else if self.tree_height > rope.tree_height {
            self.append_right(rope);
        }
        else {
            let mut rope = rope;
            mem::swap(self, &mut rope);
            self.append_left(rope);
        }
    }    
    
    
    /// Makes a copy of the rope as a string.
    /// Runs in O(N) time.
    pub fn to_string(&self) -> String {
        let mut s = String::new();

        for chunk in self.chunk_iter() {
            s.push_str(chunk);
        }
        
        return s;
    }
    
    
    /// Creates a chunk iterator for the rope
    pub fn chunk_iter<'a>(&'a self) -> RopeChunkIter<'a> {
        self.chunk_iter_at_index(0).1
    }
    
    
    /// Creates a chunk iter starting at the chunk containing the given
    /// grapheme index.  Returns the chunk and its starting grapheme index.
    pub fn chunk_iter_at_index<'a>(&'a self, index: usize) -> (usize, RopeChunkIter<'a>) {
        let mut node_stack: Vec<&'a Rope> = Vec::new();
        let mut cur_node = self;
        let mut grapheme_i = index;
        
        // Find the right rope node, and populate the stack at the same time
        loop {
            match cur_node.data {
                RopeData::Leaf(_) => {
                    node_stack.push(cur_node);
                    break;
                },
                
                RopeData::Branch(ref left, ref right) => {
                    if grapheme_i < left.grapheme_count_ {
                        node_stack.push(&(**right));
                        cur_node = &(**left);
                    }
                    else {
                        cur_node = &(**right);
                        grapheme_i -= left.grapheme_count_;
                    }
                }
            }
        }
        
        (index - grapheme_i, RopeChunkIter {node_stack: node_stack})
    }
    
    
    // TODO:
    // char_iter()
    // char_iter_at_index()
    // char_iter_between_indices()
    
    
    /// Creates an iterator at the first grapheme of the rope
    pub fn grapheme_iter<'a>(&'a self) -> RopeGraphemeIter<'a> {
        self.grapheme_iter_at_index(0)
    }
    
    
    /// Creates an iterator at the given grapheme index
    pub fn grapheme_iter_at_index<'a>(&'a self, index: usize) -> RopeGraphemeIter<'a> {
        let (grapheme_i, mut chunk_iter) = self.chunk_iter_at_index(index);
        
        // Create the grapheme iter for the current node
        let mut giter = if let Some(text) = chunk_iter.next() {
            text.as_slice().graphemes(true)
        }
        else {
            unreachable!()
        };
        
        // Get to the right spot in the iter
        for _ in grapheme_i..index {
            giter.next();
        }
        
        // Create the rope grapheme iter
        return RopeGraphemeIter {
            chunk_iter: chunk_iter,
            cur_chunk: giter,
            length: None,
        };
    }
    
    
    /// Creates an iterator that starts a pos_a and stops just before pos_b.
    pub fn grapheme_iter_between_indices<'a>(&'a self, pos_a: usize, pos_b: usize) -> RopeGraphemeIter<'a> {
        let mut iter = self.grapheme_iter_at_index(pos_a);
        iter.length = Some(pos_b - pos_a);
        return iter;
    }
    
    
    /// Creates an iterator over the lines in the rope.
    pub fn line_iter<'a>(&'a self) -> RopeLineIter<'a> {
        RopeLineIter {
            rope: self,
            li: 0,
        }
    }
    
    
    /// Creates an iterator over the lines in the rope, starting at the given
    /// line index.
    pub fn line_iter_at_index<'a>(&'a self, index: usize) -> RopeLineIter<'a> {
        RopeLineIter {
            rope: self,
            li: index,
        }
    }
    
    
    // TODO: change pos_a and pos_b to be char indices instead of grapheme
    // indices
    pub fn slice<'a>(&'a self, pos_a: usize, pos_b: usize) -> RopeSlice<'a> {
        let a = pos_a;
        let b = min(self.grapheme_count_, pos_b);
        
        RopeSlice {
            rope: self,
            start: a,
            end: b,
        }
    }
    
    
    // Creates a graphviz document of the Rope's structure, and returns
    // it as a string.  For debugging purposes.
    pub fn to_graphviz(&self) -> String {
        let mut text = String::from_str("digraph {\n");
        self.to_graphviz_recursive(&mut text, String::from_str("s"));
        text.push_str("}\n");
        return text;
    }
    
    
    //================================================================
    // Private utility functions
    //================================================================
    
    
    fn to_graphviz_recursive(&self, text: &mut String, name: String) {
        match self.data {
            RopeData::Leaf(_) => {
                text.push_str(format!("{} [label=\"cc={}\\ngc={}\\nlec={}\"];\n", name, self.char_count_, self.grapheme_count_, self.line_ending_count).as_slice());
            },
            
            RopeData::Branch(ref left, ref right) => {
                let mut lname = name.clone();
                let mut rname = name.clone();
                lname.push('l');
                rname.push('r');
                text.push_str(format!("{} [shape=box, label=\"h={}\\ncc={}\\ngc={}\\nlec={}\"];\n", name, self.tree_height, self.char_count_, self.grapheme_count_, self.line_ending_count).as_slice());
                text.push_str(format!("{} -> {{ {} {} }};\n", name, lname, rname).as_slice());
                left.to_graphviz_recursive(text, lname);
                right.to_graphviz_recursive(text, rname);
            }
        }
    }
    
    
    fn is_leaf(&self) -> bool {
        if let RopeData::Leaf(_) = self.data {
            true
        }
        else {
            false
        }
    }
    

    /// Non-recursively updates the stats of a node    
    fn update_stats(&mut self) {
        match self.data {
            RopeData::Leaf(ref text) => {
                let (cc, gc, lec) = char_grapheme_line_ending_count(text);
                self.char_count_ = cc;
                self.grapheme_count_ = gc;
                self.line_ending_count = lec;
                self.tree_height = 1;
            },
            
            RopeData::Branch(ref left, ref right) => {
                self.char_count_ = left.char_count_ + right.char_count_;
                self.grapheme_count_ = left.grapheme_count_ + right.grapheme_count_;
                self.line_ending_count = left.line_ending_count + right.line_ending_count;
                self.tree_height = max(left.tree_height, right.tree_height) + 1;
            }
        }
    }
    
    
    // TODO: change to work in terms of char indices
    fn split_recursive(&mut self, pos: usize, left: &mut Rope, right: &mut Rope) {
        match self.data {
            RopeData::Leaf(ref text) => {
                // Split the text into two new nodes
                let mut l_text = text.clone();
                let r_text = split_string_at_grapheme_index(&mut l_text, pos);
                let new_rope_l = Rope::from_string(l_text);
                let mut new_rope_r = Rope::from_string(r_text);
                
                // Append the nodes to their respective sides
                left.append(new_rope_l);
                mem::swap(right, &mut new_rope_r);
                right.append(new_rope_r);
            },
            
            RopeData::Branch(ref mut left_b, ref mut right_b) => {
                let mut l = Rope::new();
                let mut r = Rope::new();
                mem::swap(&mut **left_b, &mut l);
                mem::swap(&mut **right_b, &mut r);
                
                // Split is on left side
                if pos < l.grapheme_count_ {
                    // Append the right split to the right side
                    mem::swap(right, &mut r);
                    right.append(r);
                    
                    // Recurse
                    if let RopeData::Branch(_, ref mut new_left) = left.data {
                        if let RopeData::Branch(ref mut new_right, _) = right.data {
                            l.split_recursive(pos, new_left, new_right);
                        }
                        else {
                            l.split_recursive(pos, new_left, right);
                        }
                    }
                    else {
                        if let RopeData::Branch(ref mut new_right, _) = right.data {
                            l.split_recursive(pos, left, new_right);
                        }
                        else {
                            l.split_recursive(pos, left, right);
                        }
                    }
                }
                // Split is on right side
                else {
                    // Append the left split to the left side
                    let new_pos = pos - l.grapheme_count_;
                    left.append(l);
                    
                    // Recurse
                    if let RopeData::Branch(_, ref mut new_left) = left.data {
                        if let RopeData::Branch(ref mut new_right, _) = right.data {
                            r.split_recursive(new_pos, new_left, new_right);
                        }
                        else {
                            r.split_recursive(new_pos, new_left, right);
                        }
                    }
                    else {
                        if let RopeData::Branch(ref mut new_right, _) = right.data {
                            r.split_recursive(new_pos, left, new_right);
                        }
                        else {
                            r.split_recursive(new_pos, left, right);
                        }
                    }
                }
            },
            
        }
        
        left.rebalance();
        right.rebalance();
    }
    
    
    fn append_right(&mut self, rope: Rope) {
        if self.tree_height <= rope.tree_height || self.is_leaf() {
            let mut temp_rope = Box::new(Rope::new());
            mem::swap(self, &mut (*temp_rope));
            self.data = RopeData::Branch(temp_rope, Box::new(rope));
        }
        else if let RopeData::Branch(_, ref mut right) = self.data {
            right.append_right(rope);
        }
        
        self.update_stats();
        self.rebalance();
    }
    
    
    fn append_left(&mut self, rope: Rope) {
        if self.tree_height <= rope.tree_height || self.is_leaf() {
            let mut temp_rope = Box::new(Rope::new());
            mem::swap(self, &mut (*temp_rope));
            self.data = RopeData::Branch(Box::new(rope), temp_rope);
        }
        else if let RopeData::Branch(ref mut left, _) = self.data {
            left.append_left(rope);
        }
        
        self.update_stats();
        self.rebalance();
    }


    /// Splits a leaf node into pieces if it's too large
    // TODO: find a way to do this that's more algorithmically efficient
    // if lots of splits need to happen.  This version ends up re-scanning
    // the text quite a lot, as well as doing quite a few unnecessary
    // allocations.
    fn split_if_too_large(&mut self) {
        if self.grapheme_count_ > MAX_NODE_SIZE && self.is_leaf() {
            
            // Calculate split position and how large the left and right
            // sides are going to be
            let split_pos = self.grapheme_count_ / 2;
            let new_gc_l = split_pos;
            let new_gc_r = self.grapheme_count_ - split_pos;

            // Do the split
            let mut nl = Box::new(Rope::new());
            let mut nr = Box::new(Rope::new());
            mem::swap(self, &mut (*nl));
            if let RopeData::Leaf(ref mut text) = nl.data {
                nr.data = RopeData::Leaf(split_string_at_grapheme_index(text, split_pos));
                text.shrink_to_fit();
            }
            
            // Recursively split
            nl.grapheme_count_ = new_gc_l;
            nr.grapheme_count_ = new_gc_r;
            nl.split_if_too_large();
            nr.split_if_too_large();
            
            // Update the new left and right node's stats
            nl.update_stats();
            nr.update_stats();
            
            // Create the new branch node with the new left and right nodes
            self.data = RopeData::Branch(nl, nr);
            self.update_stats();
        }
    }
    
    
    /// Merges a non-leaf node into a leaf node if it's too small
    fn merge_if_too_small(&mut self) {
        if self.grapheme_count_ < MIN_NODE_SIZE && !self.is_leaf() {
            let mut merged_text = String::new();
            
            if let RopeData::Branch(ref mut left, ref mut right) = self.data {
                // First, recursively merge the children
                left.merge_if_too_small();
                right.merge_if_too_small();
                
                // Then put their text into merged_text
                if let RopeData::Leaf(ref mut text) = left.data {
                    mem::swap(&mut merged_text, text);
                }        
                if let RopeData::Leaf(ref mut text) = right.data {
                    merged_text.push_str(text.as_slice());
                }
            }
            
            // Make this a leaf node with merged_text as its data
            self.data = RopeData::Leaf(merged_text);
            self.tree_height = 1;
            // Don't need to update grapheme count, because it should be the
            // same as before.
        }
    }
    
    
    /// Rotates the tree under the node left
    fn rotate_left(&mut self) {
        let mut temp = Rope::new();
        
        if let RopeData::Branch(_, ref mut right) = self.data {
            mem::swap(&mut temp, &mut (**right));
            
            if let RopeData::Branch(ref mut left, _) = temp.data {   
                mem::swap(&mut (**left), &mut (**right));
            }
            else {
                panic!("Rope::rotate_left(): attempting to rotate node without branching right child.");
            }
        }
        else {
            panic!("Rope::rotate_left(): attempting to rotate leaf node.");
        }
        
        if let RopeData::Branch(ref mut left, _) = temp.data {
            mem::swap(&mut (**left), self);
            left.update_stats();
        }
        
        mem::swap(&mut temp, self);
        self.update_stats();
    }
    
    
    /// Rotates the tree under the node right
    fn rotate_right(&mut self) {
        let mut temp = Rope::new();
        
        if let RopeData::Branch(ref mut left, _) = self.data {
            mem::swap(&mut temp, &mut (**left));
            
            if let RopeData::Branch(_, ref mut right) = temp.data {   
                mem::swap(&mut (**right), &mut (**left));
            }
            else {
                panic!("Rope::rotate_right(): attempting to rotate node without branching left child.");
            }
        }
        else {
            panic!("Rope::rotate_right(): attempting to rotate leaf node.");
        }
        
        if let RopeData::Branch(_, ref mut right) = temp.data {
            mem::swap(&mut (**right), self);
            right.update_stats();
        }
        
        mem::swap(&mut temp, self);
        self.update_stats();
    }
    
    
    /// Balances the tree under this node.  Assumes that both the left and
    /// right sub-trees are themselves aleady balanced.
    /// Runs in time linear to the difference in height between the two
    /// sub-trees.  Thus worst-case is O(log N) time, and best-case is O(1)
    /// time.
    fn rebalance(&mut self) {
        let mut rot: isize = 0;
        
        if let RopeData::Branch(ref mut left, ref mut right) = self.data {
            let height_diff = (left.tree_height as isize) - (right.tree_height as isize);

            // Left side higher than right side
            if height_diff > 1 {
                let mut child_rot = false;
                if let RopeData::Branch(ref lc, ref rc) = left.data {
                    if lc.tree_height < rc.tree_height {
                        child_rot = true;
                    }
                }
                
                if child_rot {
                    left.rotate_left();
                }
                
                rot = 1;
            }
            // Right side higher then left side
            else if height_diff < -1 {
                let mut child_rot = false;
                if let RopeData::Branch(ref lc, ref rc) = right.data {
                    if lc.tree_height > rc.tree_height {
                        child_rot = true;
                    }
                }
                
                if child_rot {
                    right.rotate_right();
                }
                
                rot = -1;
            }
        }
        
        if rot == 1 {
            self.rotate_right();
            if let RopeData::Branch(_, ref mut right) = self.data {
                right.rebalance();
            }
        }
        else if rot == -1 {
            self.rotate_left();
            if let RopeData::Branch(ref mut left, _) = self.data {
                left.rebalance();
            }
        }
        
        self.update_stats();
    }
    
    
    /// Tests if the rope adheres to the AVL balancing invariants.
    fn is_balanced(&self) -> bool {
        match self.data {
            RopeData::Leaf(_) => {
                return true;
            },
            
            RopeData::Branch(ref left, ref right) => {
                let mut diff = left.tree_height as isize - right.tree_height as isize;
                diff = if diff < 0 {-diff} else {diff};
                return (diff < 2) && left.is_balanced() && right.is_balanced();
            }
        }
    }
}


// Direct indexing to graphemes in the rope
// TODO: change to work in terms of chars, since they're the atomic unit of
// a Rope.
impl Index<usize> for Rope {
    type Output = str;
    
    fn index<'a>(&'a self, index: &usize) -> &'a str {
        if *index >= self.grapheme_count() {
            panic!("Rope::Index: attempting to fetch grapheme that outside the bounds of the text.");
        }
        
        match self.data {
            RopeData::Leaf(ref text) => {
                let mut i: usize = 0;
                for g in text.graphemes(true) {
                    if i == *index {
                        return &g;
                    }
                    i += 1;
                }
                unreachable!();
            },
            
            RopeData::Branch(ref left, ref right) => {
                if *index < left.grapheme_count() {
                    return &left[*index];
                }
                else {
                    return &right[*index - left.grapheme_count()];
                }
            },
        }
    }
}




//=============================================================
// Rope iterators
//=============================================================

/// An iterator over a rope's string chunks
pub struct RopeChunkIter<'a> {
    node_stack: Vec<&'a Rope>,
}

impl<'a> Iterator for RopeChunkIter<'a> {
    type Item = &'a str;
    
    fn next(&mut self) -> Option<&'a str> {
        if let Some(next_chunk) = self.node_stack.pop() {
            loop {
                if let Option::Some(node) = self.node_stack.pop() {
                    match node.data {
                        RopeData::Leaf(_) => {
                            self.node_stack.push(node);
                            break;
                        },
                      
                        RopeData::Branch(ref left, ref right) => {
                            self.node_stack.push(&(**right));
                            self.node_stack.push(&(**left));
                            continue;
                        }
                    }
                }
                else {
                    break;
                }
            }
            
            if let RopeData::Leaf(ref text) = next_chunk.data {
                return Some(text.as_slice());
            }
            else {
                unreachable!();
            }
        }
        else {
            return None;
        }
    }
}


// TODO:
// RopeCharIter


/// An iterator over a rope's graphemes
pub struct RopeGraphemeIter<'a> {
    chunk_iter: RopeChunkIter<'a>,
    cur_chunk: Graphemes<'a>,
    length: Option<usize>,
}


impl<'a> Iterator for RopeGraphemeIter<'a> {
    type Item = &'a str;
    
    fn next(&mut self) -> Option<&'a str> {
        if let Some(ref mut l) = self.length {
            if *l == 0 {
                return None;
            }
        }
        
        loop {
            if let Some(g) = self.cur_chunk.next() {
                if let Some(ref mut l) = self.length {
                    *l -= 1;
                }
                return Some(g);
            }
            else {   
                if let Some(s) = self.chunk_iter.next() {
                    self.cur_chunk = s.graphemes(true);
                    continue;
                }
                else {
                    return None;
                }
            }
        }
    }
}



/// An iterator over a rope's lines, returned as RopeSlice's
pub struct RopeLineIter<'a> {
    rope: &'a Rope,
    li: usize,
}


impl<'a> Iterator for RopeLineIter<'a> {
    type Item = RopeSlice<'a>;

    fn next(&mut self) -> Option<RopeSlice<'a>> {
        if self.li >= self.rope.line_count() {
            return None;
        }
        else {
            let a = self.rope.line_index_to_grapheme_index(self.li);
            let b = if self.li+1 < self.rope.line_count() {
                self.rope.line_index_to_grapheme_index(self.li+1)
            }
            else {
                self.rope.grapheme_count()
            };
            
            self.li += 1;
            
            return Some(self.rope.slice(a, b));
        }
    }
}




//=============================================================
// Rope slice
//=============================================================

/// An immutable slice into a Rope
pub struct RopeSlice<'a> {
    rope: &'a Rope,
    start: usize,
    end: usize,
}


impl<'a> RopeSlice<'a> {
    pub fn char_count(&self) -> usize {
        unimplemented!()
    }
    

    pub fn grapheme_count(&self) -> usize {
        self.end - self.start
    }
    
    
    // TODO:
    // char_iter()
    // char_iter_at_index()
    // char_iter_between_indices()
    
    
    pub fn grapheme_iter(&self) -> RopeGraphemeIter<'a> {
        self.rope.grapheme_iter_between_indices(self.start, self.end)
    }
    
    pub fn grapheme_iter_at_index(&self, pos: usize) -> RopeGraphemeIter<'a> {
        let a = min(self.end, self.start + pos);
        
        self.rope.grapheme_iter_between_indices(a, self.end)
    }
    
    pub fn grapheme_iter_between_indices(&self, pos_a: usize, pos_b: usize) -> RopeGraphemeIter<'a> {
        let a = min(self.end, self.start + pos_a);
        let b = min(self.end, self.start + pos_b);
        
        self.rope.grapheme_iter_between_indices(a, b)
    }
    
    
    pub fn char_at_index(&self, index: usize) -> char {
        unimplemented!()
    }
    
    pub fn grapheme_at_index(&self, index: usize) -> &'a str {
        &self.rope[self.start+index]
    }
    
    
    // TODO: change to work in terms of char indices instead of
    // grapheme indices
    pub fn slice(&self, pos_a: usize, pos_b: usize) -> RopeSlice<'a> {
        let a = min(self.end, self.start + pos_a);
        let b = min(self.end, self.start + pos_b);
        
        RopeSlice {
            rope: self.rope,
            start: a,
            end: b,
        }
    }
}
