// Copyright 2026 TH3AL3X
// SPDX-License-Identifier: GPL-3.0-only
//! Persistent desktop cells. Moving an icon changes this sidecar, never the file itself.
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

pub fn internal_drop(directory: &Path, paths: &[PathBuf]) -> bool {
    !paths.is_empty() && paths.iter().all(|p| p.parent() == Some(directory))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Cell { pub row: usize, pub col: usize }

#[derive(Clone, Copy, Debug)]
pub struct Grid {
    pub cols: usize, pub rows: usize,
    pub step_x: f32, pub step_y: f32, pub padding: f32,
    pub item_width: f32, pub item_height: f32,
}
impl Grid {
    pub fn snap(self, x: f32, y: f32) -> Cell {
        let coordinate = |v: f32, step: f32, size: f32, limit: usize| {
            if !v.is_finite() { return 0; }
            (((v-self.padding-size/2.0)/step).round().max(0.0) as usize).min(limit.saturating_sub(1))
        };
        Cell { row: coordinate(y,self.step_y,self.item_height,self.rows),
               col: coordinate(x,self.step_x,self.item_width,self.cols) }
    }
    fn from_index(self, index: usize) -> Cell {
        let page = index/(self.cols*self.rows);
        let within = index%(self.cols*self.rows);
        Cell { row: page*self.rows+within%self.rows, col: within/self.rows }
    }
}

#[derive(Debug)]
pub struct Layout {
    pub enabled: bool,
    pub positions: BTreeMap<String, Cell>,
    path: Option<PathBuf>,
}
impl Layout {
    pub fn load(path: PathBuf) -> Self {
        let enabled=path.parent().and_then(|p| std::fs::read_to_string(p.join("enabled")).ok())
            .is_none_or(|s| s.trim()!="false");
        let mut layout=Self { enabled,positions:BTreeMap::new(),path:Some(path.clone()) };
        if let Ok(text)=std::fs::read_to_string(&path) {
            for line in text.lines().take(100_000) {
                let fields:Vec<_>=line.split('\t').collect();
                if fields.len()!=3 { continue; }
                if let (Ok(row),Ok(col),Some(key))=(fields[0].parse::<usize>(),fields[1].parse::<usize>(),decode(fields[2])) {
                    if row<1_000_000 && col<1_000_000 { layout.positions.insert(key,Cell{row,col}); }
                }
            }
        }
        layout
    }
    pub fn resolve(&mut self, keys: &[String], grid: Grid) -> Vec<Cell> {
        let mut occupied=BTreeSet::new();
        let max_rows=grid.rows*keys.len().max(1).div_ceil(grid.cols*grid.rows);
        let mut cells:Vec<_>=keys.iter().map(|key| {
            self.positions.get(key).copied().filter(|cell| cell.col<grid.cols && cell.row<max_rows && occupied.insert(*cell))
        }).collect();
        let mut changed=false;
        for (key,cell) in keys.iter().zip(&mut cells) {
            if cell.is_none() {
                let free=(0..).map(|i|grid.from_index(i)).find(|c|!occupied.contains(c)).unwrap();
                occupied.insert(free); *cell=Some(free);
            }
            let cell=cell.unwrap();
            if self.positions.get(key)!=Some(&cell) { self.positions.insert(key.clone(),cell);changed=true; }
        }
        if changed { self.save(); }
        cells.into_iter().flatten().collect()
    }
    pub fn move_group(&mut self, keys: &[String], all_keys: &[String], target: Cell, grid: Grid) {
        if keys.is_empty() { return; }
        let moving:BTreeSet<_>=keys.iter().cloned().collect();
        let mut occupied:BTreeSet<_>=all_keys.iter().filter(|k|!moving.contains(*k))
            .filter_map(|k|self.positions.get(k).copied()).collect();
        let origin=Cell {
            row:keys.iter().filter_map(|k|self.positions.get(k).map(|c|c.row)).min().unwrap_or(0),
            col:keys.iter().filter_map(|k|self.positions.get(k).map(|c|c.col)).min().unwrap_or(0),
        };
        let offsets:Vec<_>=keys.iter().enumerate().map(|(i,k)|self.positions.get(k)
            .map(|c|Cell{row:c.row-origin.row,col:c.col-origin.col})
            .unwrap_or(Cell{row:i,col:0})).collect();
        let width=offsets.iter().map(|c|c.col+1).max().unwrap_or(1);
        let height=offsets.iter().map(|c|c.row+1).max().unwrap_or(1);
        let mut anchors:Vec<_>=(0..grid.rows).flat_map(|row|(0..grid.cols).map(move |col|Cell{row,col})).collect();
        anchors.sort_by_key(|c|(c.row.abs_diff(target.row)+c.col.abs_diff(target.col),c.col,c.row));
        let anchor=anchors.iter().copied().find(|c| c.col+width<=grid.cols && c.row+height<=grid.rows &&
            offsets.iter().all(|offset|!occupied.contains(&Cell{row:c.row+offset.row,col:c.col+offset.col})));
        if let Some(anchor)=anchor {
            for (key,offset) in keys.iter().zip(offsets) {
                self.positions.insert(key.clone(),Cell{row:anchor.row+offset.row,col:anchor.col+offset.col});
            }
        } else {
            // A fragmented/full desktop must never overlap or discard icons.
            for key in keys {
                let cell=anchors.iter().copied().find(|c|!occupied.contains(c))
                    .unwrap_or_else(||(0..).map(|i|grid.from_index(i)).find(|c|!occupied.contains(c)).unwrap());
                occupied.insert(cell);self.positions.insert(key.clone(),cell);
            }
        }
        self.save();
    }
    pub fn rename(&mut self, old: &str, new: &str) {
        if let Some(cell) = self.positions.remove(old) {
            self.positions.insert(new.to_owned(), cell);
            self.save();
        }
    }
    fn save(&self) {
        let Some(path)=&self.path else{return};
        let Some(parent)=path.parent() else{return};
        let text=self.positions.iter().map(|(key,c)|format!("{}\t{}\t{}\n",c.row,c.col,encode(key))).collect::<String>();
        let result=(||->std::io::Result<()> {
            std::fs::create_dir_all(parent)?;
            let temporary=path.with_extension("tmp");
            std::fs::write(&temporary,text)?;
            std::fs::rename(temporary,path)
        })();
        if let Err(error)=result { eprintln!("Could not save desktop layout: {error}"); }
    }
}
fn encode(s:&str)->String { s.as_bytes().iter().map(|b|format!("{b:02x}")).collect() }
fn decode(s:&str)->Option<String> {
    if s.len()%2!=0 {return None;}
    let bytes=(0..s.len()).step_by(2).map(|i|u8::from_str_radix(s.get(i..i+2)?,16).ok()).collect::<Option<Vec<_>>>()?;
    String::from_utf8(bytes).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn grid()->Grid {Grid{cols:4,rows:3,step_x:100.,step_y:100.,padding:0.,item_width:100.,item_height:100.}}
    fn layout()->Layout {Layout{enabled:true,positions:BTreeMap::new(),path:None}}
    fn keys(names:&[&str])->Vec<String>{names.iter().map(|s|s.to_string()).collect()}
    #[test] fn only_same_directory_drags_are_repositioned_without_file_operations() {
        let desktop=Path::new("/home/test/Escritorio");
        assert!(internal_drop(desktop,&[desktop.join("a"),desktop.join("b")]));
        assert!(!internal_drop(desktop,&[]));
        assert!(!internal_drop(desktop,&[PathBuf::from("/home/test/Documentos/a")]));
        assert!(!internal_drop(desktop,&[desktop.join("a"),PathBuf::from("/tmp/external")]));
    }
    #[test] fn renaming_keeps_the_assigned_cell() {
        let mut l=layout();l.positions.insert("old".into(),Cell{row:2,col:3});
        l.rename("old","new");
        assert_eq!(l.resolve(&keys(&["new"]),grid()),vec![Cell{row:2,col:3}]);
        assert!(!l.positions.contains_key("old"));
    }
    #[test] fn new_files_do_not_displace_saved_icons() {
        let mut l=layout();l.positions.insert("old".into(),Cell{row:0,col:0});
        let c=l.resolve(&keys(&["new","old"]),grid());
        assert_eq!(c[1],Cell{row:0,col:0});assert_ne!(c[0],c[1]);
    }
    #[test] fn collision_preserves_other_icons_and_group_shape() {
        let mut l=layout();let names=keys(&["a","b","fixed"]);l.resolve(&names,grid());
        let fixed=l.positions["fixed"];
        l.move_group(&keys(&["a","b"]),&names,fixed,grid());
        assert_eq!(l.positions["fixed"],fixed);
        assert_eq!(l.positions["b"].row,l.positions["a"].row+1);
        assert_eq!(l.positions.values().collect::<BTreeSet<_>>().len(),3);
    }
    #[test] fn smaller_monitor_and_full_grid_never_lose_or_overlap_items() {
        let mut l=layout();let names=(0..30).map(|i|i.to_string()).collect::<Vec<_>>();
        l.positions.insert("0".into(),Cell{row:999,col:999});
        let positions=l.resolve(&names,grid());
        assert_eq!(positions.len(),30);assert_eq!(positions.iter().collect::<BTreeSet<_>>().len(),30);
        assert!(positions.iter().all(|c|c.col<4));
        l.move_group(&keys(&["0","1"]),&names,Cell{row:0,col:0},grid());
        assert_eq!(l.positions.values().collect::<BTreeSet<_>>().len(),30);
    }
    #[test] fn snap_handles_edges_and_invalid_coordinates() {
        assert_eq!(grid().snap(246.,149.),Cell{row:1,col:2});
        assert_eq!(grid().snap(-200.,f32::NAN),Cell{row:0,col:0});
        assert_eq!(grid().snap(9999.,9999.),Cell{row:2,col:3});
    }
    #[test] fn restart_preserves_names_with_spaces_tabs_and_accents() {
        let path=std::env::temp_dir().join(format!("cosmic-layout-test-{}.cells",std::process::id()));
        let mut l=Layout::load(path.clone());let names=keys(&["Imágenes\tcon espacios","b"]);
        l.resolve(&names,grid());l.move_group(&names[..1],&names,Cell{row:2,col:3},grid());
        let mut reloaded=Layout::load(path.clone());let positions=reloaded.resolve(&names,grid());
        assert_eq!(positions[0],Cell{row:2,col:3});std::fs::remove_file(path).unwrap();
    }
}
