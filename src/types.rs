use std::{
    ops::DerefMut,
    ptr::{addr_of, addr_of_mut},
};

use rand::{seq::SliceRandom, Rng};
use wasm_bindgen::prelude::wasm_bindgen;

#[derive(Copy, Clone, PartialEq)]
#[wasm_bindgen]
pub struct RCCoords {
    pub r: usize,
    pub c: usize,
}

#[derive(Copy, Clone, PartialEq)]
#[wasm_bindgen]
pub struct GNCoords {
    pub g: usize,
    pub n: usize,
}

impl RCCoords {
    pub fn from_idx(idx: usize) -> RCCoords {
        RCCoords {
            r: idx / 9,
            c: idx % 9,
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
#[wasm_bindgen]
/// 兼容上述两种坐标系
pub struct Coords {
    pub r: usize,
    pub c: usize,
    pub g: usize,
    pub n: usize,
}

impl From<Coords> for RCCoords {
    fn from(coords: Coords) -> Self {
        let Coords { r, c, g: _, n: _ } = coords;
        RCCoords { r, c }
    }
}

impl From<Coords> for GNCoords {
    fn from(coords: Coords) -> Self {
        let Coords { r: _, c: _, g, n } = coords;
        GNCoords { g, n }
    }
}

impl From<RCCoords> for Coords {
    fn from(rc: RCCoords) -> Self {
        let RCCoords { r, c } = rc;
        let GNCoords { g, n } = rc.into();
        Coords { r, c, g, n }
    }
}

impl From<GNCoords> for Coords {
    fn from(gn: GNCoords) -> Self {
        let RCCoords { r, c } = gn.into();
        let GNCoords { g, n } = gn;
        Coords { r, c, g, n }
    }
}

impl From<GNCoords> for RCCoords {
    fn from(gn: GNCoords) -> Self {
        let GNCoords { g, n } = gn;
        RCCoords {
            r: (g / 3 * 3 + n / 3),
            c: (g % 3 * 3 + n % 3),
        }
    }
}

impl From<RCCoords> for GNCoords {
    fn from(rc: RCCoords) -> Self {
        let RCCoords { r, c } = rc;
        GNCoords {
            g: (r / 3 * 3 + c / 3),
            n: (r % 3 * 3 + c % 3),
        }
    }
}

#[derive(Default, Copy, Clone, PartialEq, Eq, Hash)]
#[wasm_bindgen]
pub struct Candidate(u16);

impl Candidate {
    pub const FULL: Candidate = Candidate(0x1FF);

    pub fn get_unique_candidate(&self) -> Option<u8> {
        const fn bit_to_num() -> [u16; 9] {
            let mut result = [0; 9];
            let mut i = 0;
            while i < 9 {
                result[i] = 0x001 << i;
                i += 1;
            }
            result
        }
        const BIT_TO_NUM: [u16; 9] = bit_to_num();

        BIT_TO_NUM
            .iter()
            .position(|p| *p == self.0)
            .map(|r| r as u8)
    }

    pub fn add(&mut self, v: u8) {
        self.0 &= 0x001 << v as u16;
    }

    pub fn remove(&mut self, v: u8) {
        self.0 &= !(0x001 << v as u16);
    }

    pub fn contains(&self, v: u8) -> bool {
        self.0 & (0x001 << v as u16) != 0
    }

    pub fn len(&self) -> u32 {
        self.0.count_ones()
    }

    pub fn union(&self, other: &Candidate) -> Candidate {
        Candidate(self.0 | other.0)
    }

    pub fn intersect(&self, other: &Candidate) -> Candidate {
        Candidate(self.0 & other.0)
    }

    pub fn subtract(&self, other: &Candidate) -> Candidate {
        Candidate(self.0 & !other.0)
    }

    pub fn to_vec(&self) -> Vec<u8> {
        let mut values = Vec::new();
        for i in 0..9 {
            if self.0 & (0x001 << i) != 0 {
                values.push(i);
            }
        }
        values
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
#[wasm_bindgen]
pub enum CellStatus {
    // 固定数值
    FIXED,
    // 草稿，未填值
    DRAFT,
    // 用户的解答，已填值，此时drafts数组的内容将被忽略
    SOLVE,
}

#[derive(Clone, Copy)]
#[wasm_bindgen]
pub struct Cell {
    pub coords: Coords,
    pub status: CellStatus,
    pub candidates: Candidate,
    pub value: Option<u8>, // None 0-9
                           // pub answer: u8, // The Answer Of This Cell
}

impl std::fmt::Debug for Candidate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for i in 0..9 {
            if self.0 & (0x001 << i) != 0 {
                write!(f, "{}", i + 1)?;
            }
        }
        write!(f, "")
    }
}

impl std::fmt::Debug for RCCoords {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "R{}C{}", self.r + 1, self.c + 1)
    }
}

impl std::fmt::Debug for GNCoords {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "G{}N{}", self.g + 1, self.n + 1)
    }
}

impl std::fmt::Debug for Coords {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "R{}C{}G{}N{}",
            self.r + 1,
            self.c + 1,
            self.g + 1,
            self.n + 1
        )
    }
}

impl std::fmt::Debug for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.coords)?;
        match self.status {
            CellStatus::FIXED | CellStatus::SOLVE => write!(f, "{:?};", self.value),
            CellStatus::DRAFT => {
                write!(f, "D{:?};", self.candidates)
            }
        }
    }
}

/// 数组本体
#[derive(Clone)]
#[wasm_bindgen]
pub struct Sudoku {
    cells: [Cell; 81],
}

impl Sudoku {
    pub fn get_cell_mut_by_rc(&mut self, rc: RCCoords) -> &mut Cell {
        &mut self.cells[rc.r * 9 + rc.c]
    }

    pub fn get_cell_ref_by_rc(&self, rc: RCCoords) -> &Cell {
        &self.cells[rc.r * 9 + rc.c]
    }

    pub fn get_cell_mut_ptr_by_rc(&mut self, rc: RCCoords) -> *mut Cell {
        &mut self.cells[rc.r * 9 + rc.c]
    }

    pub fn get_cell_mut_by_gn(&mut self, gn: GNCoords) -> &mut Cell {
        let RCCoords { r, c } = gn.into();
        &mut self.cells[r * 9 + c]
    }

    pub fn get_cell_ref_by_gn(&self, gn: GNCoords) -> &Cell {
        let RCCoords { r, c } = gn.into();
        &self.cells[r * 9 + c]
    }

    pub fn get_cell_mut_ptr_by_gn(&mut self, gn: GNCoords) -> *mut Cell {
        let RCCoords { r, c } = gn.into();
        &mut self.cells[r * 9 + c]
    }

    pub fn get_cell_mut_by_coords(&mut self, coords: Coords) -> &mut Cell {
        let Coords { r, c, g: _, n: _ } = coords;
        &mut self.cells[r * 9 + c]
    }

    pub fn get_cell_ref_by_coords(&self, coords: Coords) -> &Cell {
        let Coords { r, c, g: _, n: _ } = coords;
        &self.cells[r * 9 + c]
    }

    pub fn get_cell_mut_ptr_by_coords(&mut self, coords: Coords) -> *mut Cell {
        let Coords { r, c, g: _, n: _ } = coords;
        &mut self.cells[r * 9 + c]
    }

    // 如果格子的内容有冲突，也说明有错误，可以不继续推理下去了
    pub fn find_conflict(&self) -> Option<Vec<(&Cell, &Cell)>> {
        let mut ret: Vec<(&Cell, &Cell)> = vec![];
        for r in 0..9 {
            for c in 0..9 {
                let rc = RCCoords { r, c };
                let p_cell = self.get_cell_ref_by_rc(rc);
                if p_cell.status == CellStatus::FIXED || p_cell.status == CellStatus::SOLVE {
                    let v = p_cell.value;
                    let g = p_cell.coords.g;
                    let n = p_cell.coords.n;
                    for r_iter in (r + 1)..9 {
                        let tmp = self.get_cell_ref_by_rc(RCCoords { r: r_iter, c });
                        if (tmp.value == v)
                            && (tmp.status == CellStatus::FIXED || tmp.status == CellStatus::SOLVE)
                        {
                            ret.push((p_cell, tmp));
                        }
                    }
                    for c_iter in (c + 1)..9 {
                        let tmp = self.get_cell_ref_by_rc(RCCoords { r, c: c_iter });
                        if (tmp.value == v)
                            && (tmp.status == CellStatus::FIXED || tmp.status == CellStatus::SOLVE)
                        {
                            ret.push((p_cell, tmp));
                        }
                    }
                    for n_iter in (n + 1)..9 {
                        let tmp = self.get_cell_ref_by_gn(GNCoords { g, n: n_iter });
                        if (tmp.value == v)
                            && (tmp.status == CellStatus::FIXED || tmp.status == CellStatus::SOLVE)
                        {
                            ret.push((p_cell, tmp));
                        }
                    }
                }
            }
        }
        if !ret.is_empty() {
            Some(ret)
        } else {
            None
        }
    }

    // 仅在初始化时使用，补充所有可能的草稿数
    fn fill_drafts(&mut self) {
        for r in 0..9 {
            for c in 0..9 {
                let cell = self.get_cell_mut_by_rc(RCCoords { r, c });
                if cell.status == CellStatus::DRAFT {
                    cell.candidates = Candidate::FULL;
                }
            }
        }
        for r in 0..9 {
            for c in 0..9 {
                let rc = RCCoords { r, c };
                let &Cell {
                    status,
                    value,
                    coords,
                    ..
                } = self.get_cell_ref_by_rc(rc);
                if status == CellStatus::FIXED {
                    let g = coords.g;
                    let value = value.unwrap();
                    for i in 0..9 {
                        self.get_cell_mut_by_rc(RCCoords { r: i, c })
                            .candidates
                            .remove(value);
                        self.get_cell_mut_by_rc(RCCoords { r, c: i })
                            .candidates
                            .remove(value);
                        self.get_cell_mut_by_gn(GNCoords { g, n: i })
                            .candidates
                            .remove(value);
                    }
                }
            }
        }
    }

    // 打印数独，用特殊效果显示草稿、固定值、填写值
    pub fn print(&self) {
        const TOP_BORDER: &str = "╔═══╤═══╤═══╦═══╤═══╤═══╦═══╤═══╤═══╗";
        const BOTTOM_BORDER: &str = "╚═══╧═══╧═══╩═══╧═══╧═══╩═══╧═══╧═══╝";
        const MIDDLE_BORDER: &str = "╠═══╪═══╪═══╬═══╪═══╪═══╬═══╪═══╪═══╣";
        const SUB_BORDER: &str = "╟───┼───┼───╫───┼───┼───╫───┼───┼───╢";

        fn format_draft(p: &Cell, m: usize) -> String {
            (0..3)
                .map(|n| {
                    if p.candidates.0 & (0x001 << m * 3 + n) != 0 {
                        (m * 3 + n + 1).to_string()
                    } else {
                        " ".to_string()
                    }
                })
                .collect()
        }

        fn format_fixed(p: &Cell, m: usize) -> String {
            match m {
                0 => "\\ /".to_string(),
                1 => format!(" {} ", p.value.unwrap() + 1),
                _ => "/ \\".to_string(),
            }
        }

        fn format_solve(p: &Cell, m: usize) -> String {
            match m {
                0 => "***".to_string(),
                1 => format!("*{}*", p.value.unwrap() + 1),
                _ => "***".to_string(),
            }
        }

        println!("{}", TOP_BORDER);

        for r in 0..9 {
            for m in 0..3 {
                let mut line = String::from("║");
                for c in 0..9 {
                    let p = &self.cells[r * 9 + c];
                    line += &match p.status {
                        CellStatus::DRAFT => format_draft(p, m),
                        CellStatus::FIXED => format_fixed(p, m),
                        CellStatus::SOLVE => format_solve(p, m),
                    };
                    line += if c % 3 == 2 { "║" } else { "│" };
                }
                println!("{}", line);
            }

            if r == 8 {
                println!("{}", BOTTOM_BORDER);
            } else if r % 3 == 2 {
                println!("{}", MIDDLE_BORDER);
            } else {
                println!("{}", SUB_BORDER);
            }
        }
    }

    // TODO: Backtrace Solve
    // fn is_unique(&self) -> bool {
    //     let mut solution_count = 0;
    //     let mut solver = self.clone();
    //     solver.backtrack_solve(&mut solution_count);
    //     solution_count == 1
    // }

    // fn backtrack_solve(&mut self, count: &mut u32) -> bool {
    //     // 实现带计数器的回溯算法...
    // }
    fn self_solve_field(field: &mut Sudoku, solutions: &mut Vec<Sudoku>) -> bool {
        fn is_valid(r: usize, c: usize, v: Option<u8>, field: &Sudoku) -> bool {
            let GNCoords { g, n: _ } = RCCoords { r, c }.into();
            for i in 0..9 {
                if field.get_cell_ref_by_rc(RCCoords { r, c: i }).value == v {
                    return false;
                }
                if field.get_cell_ref_by_rc(RCCoords { r: i, c }).value == v {
                    return false;
                }
                if field.get_cell_ref_by_gn(GNCoords { g, n: i }).value == v {
                    return false;
                }
            }
            true
        }
        unsafe {
            // 找到第一个草稿状态的单元格
            for r in 0..9 {
                for c in 0..9 {
                    let idx = r * 9 + c;
                    let cell: *mut Cell = core::ptr::addr_of_mut!(field.cells[idx]);
                    if (*cell).status == CellStatus::DRAFT {
                        for &num in &(*cell).candidates.to_vec() {
                            if is_valid(r, c, Some(num), &field) {
                                (*cell).value = Some(num);
                                (*cell).status = CellStatus::SOLVE;

                                if Self::self_solve_field(field, solutions) {
                                    solutions.push(field.clone());
                                    if solutions.len() >= 2 {
                                        return true;
                                    }
                                }

                                // 回溯
                                (*cell).status = CellStatus::DRAFT;
                                (*cell).value = None; // 重置值
                            }
                        }
                        return false; // 如果没有找到有效的数字，则返回false
                    }
                }
            }
        }

        true // 如果所有单元格都已解决，则返回true
    }

    pub fn sovle(&self) -> Vec<Sudoku> {
        let mut field = self.clone();
        let mut solutions: Vec<Sudoku> = Vec::new();

        Self::self_solve_field(&mut field, &mut solutions);

        solutions
    }

    // 以下是常见的遍历手段

    /// 遍历所有单元格
    pub fn collect_all_drafts_cells(&self) -> Vec<&Cell> {
        self.cells
            .iter()
            .filter(|&p| p.status == CellStatus::DRAFT)
            .collect::<Vec<&Cell>>()
    }

    /// 按行遍历草稿单元格
    pub fn iter_all_drafts_cells_by_rc(&self) -> <Vec<Vec<&Cell>> as IntoIterator>::IntoIter {
        (0..9)
            .into_iter()
            .map(|r| {
                (0..9)
                    .into_iter()
                    .map(|c| self.get_cell_ref_by_rc(RCCoords { r, c }))
                    .filter(|&p| (*p).status == CellStatus::DRAFT)
                    .collect()
            })
            .collect::<Vec<Vec<&Cell>>>()
            .into_iter()
    }

    /// 按列遍历草稿单元格
    pub fn iter_all_drafts_cells_by_cr(&self) -> <Vec<Vec<&Cell>> as IntoIterator>::IntoIter {
        (0..9)
            .into_iter()
            .map(|c| {
                (0..9)
                    .into_iter()
                    .map(|r| self.get_cell_ref_by_rc(RCCoords { r, c }))
                    .filter(|&p| (*p).status == CellStatus::DRAFT)
                    .collect()
            })
            .collect::<Vec<Vec<&Cell>>>()
            .into_iter()
    }

    /// 按宫遍历草稿单元格
    pub fn iter_all_drafts_cells_by_gn(&self) -> <Vec<Vec<&Cell>> as IntoIterator>::IntoIter {
        (0..9)
            .into_iter()
            .map(|g| {
                (0..9)
                    .into_iter()
                    .map(|n| self.get_cell_ref_by_gn(GNCoords { g, n }))
                    .filter(|&p| (*p).status == CellStatus::DRAFT)
                    .collect()
            })
            .collect::<Vec<Vec<&Cell>>>()
            .into_iter()
    }

    /// 在指定行按列遍历单元格
    pub fn collect_all_drafts_cells_in_r(&self, r: usize) -> Vec<&Cell> {
        (0..9)
            .into_iter()
            .map(|c| self.get_cell_ref_by_rc(RCCoords { r, c }))
            .filter(|&p| (*p).status == CellStatus::DRAFT)
            .collect()
    }

    /// 在指定列按行遍历单元格
    pub fn collect_all_drafts_cells_in_c(&self, c: usize) -> Vec<&Cell> {
        (0..9)
            .into_iter()
            .map(|r| self.get_cell_ref_by_rc(RCCoords { r, c }))
            .filter(|&p| (*p).status == CellStatus::DRAFT)
            .collect()
    }

    /// 在指定宫按序遍历单元格
    pub fn collect_all_drafts_cells_in_g(&self, g: usize) -> Vec<&Cell> {
        (0..9)
            .into_iter()
            .map(|n| self.get_cell_ref_by_gn(GNCoords { g, n }))
            .filter(|&p| (*p).status == CellStatus::DRAFT)
            .collect()
    }

    /// 按行遍历包含指定V的草稿单元格
    pub fn iter_all_drafts_cells_by_rc_contains_v(
        &self,
        v: u8,
    ) -> <Vec<Vec<&Cell>> as IntoIterator>::IntoIter {
        (0..9)
            .into_iter()
            .map(|r| {
                (0..9)
                    .into_iter()
                    .map(|c| self.get_cell_ref_by_rc(RCCoords { r, c }))
                    .filter(|&p| (*p).status == CellStatus::DRAFT && (*p).candidates.contains(v))
                    .collect()
            })
            .collect::<Vec<Vec<&Cell>>>()
            .into_iter()
    }

    /// 给定一个坐标和值，根据坐标遍历同一行、同一列、同一宫所有含有这个值的单元格
    pub fn collect_all_drafts_coords_by_coords_and_value(
        &self,
        coords: Coords,
        value: u8,
    ) -> Vec<Coords> {
        let Coords { r, c, g, n: _ } = coords;

        let mut coords = vec![];

        for i in 0..9 {
            let p_cell = self.get_cell_ref_by_rc(RCCoords { r: i, c });
            if p_cell.status == CellStatus::DRAFT && p_cell.candidates.contains(value) {
                coords.push(p_cell.coords);
            }

            let p_cell = self.get_cell_ref_by_rc(RCCoords { r, c: i });
            if p_cell.coords.c != c
                && p_cell.status == CellStatus::DRAFT
                && p_cell.candidates.contains(value)
            {
                coords.push(p_cell.coords);
            }

            let p_cell = self.get_cell_ref_by_gn(GNCoords { g, n: i });
            if p_cell.coords.r != r
                && p_cell.coords.c != c
                && p_cell.status == CellStatus::DRAFT
                && p_cell.candidates.contains(value)
            {
                coords.push(p_cell.coords);
            }
        }

        coords
    }

    /// 检查是否都填写完毕了
    pub fn check_if_finish(&self) -> bool {
        self.collect_all_drafts_cells().is_empty()
    }
}

#[wasm_bindgen]
pub enum Difficulty {
    EASY,
    NORMAL,
    MIDIUM,
    HARD,
    EXPERT,
}

impl Difficulty {
    fn empty_cells(&self) -> (usize, usize) {
        match self {
            Self::EASY => (35, 40),
            Self::NORMAL => (40, 45),
            Self::MIDIUM => (45, 50),
            Self::HARD => (50, 55),
            Self::EXPERT => (55, 64), // 81-17=64
        }
    }

    // fn allowed_techniques(&self) -> Vec<SolverTechnique> {
    //     match self {
    //         Self::Easy => vec![BASIC_SINGLES],
    //         // ...
    //     }
    // }
}

// #[wasm_bindgen]
impl Sudoku {
    // 从字符串初始化数独，要求输入字符串长度必须为81，且仅为0-9的数字
    pub fn initial_by_string(input: String) -> Result<Sudoku, String> {
        if input.len() != 81 {
            return Err("Invalid String Length.".to_string());
        }

        let mut field: Sudoku = unsafe {
            let mut field = std::mem::MaybeUninit::<Sudoku>::uninit();
            let p_field: *mut Sudoku = field.as_mut_ptr();
            let p_cell: *mut Cell = (*p_field).cells.as_mut_ptr();

            for (index, item) in input.chars().enumerate() {
                let tmp = item.to_digit(10).expect("Invalid Character.");
                let rc = RCCoords::from_idx(index);
                let coords = rc.into();
                let status = if tmp == 0 {
                    CellStatus::DRAFT
                } else {
                    CellStatus::FIXED
                };
                let value = if tmp == 0 {
                    None
                } else {
                    Some((tmp - 1) as u8)
                };
                std::ptr::write(
                    p_cell.offset(index as isize),
                    Cell {
                        coords,
                        status,
                        candidates: Candidate::FULL,
                        value,
                    },
                );
            }

            field.assume_init()
        };

        field.fill_drafts();

        Ok(field)
    }

    // 采用洗牌算法+随机挖空生成随机数独
    pub fn new(difficulty: Difficulty) -> Self {
        fn create_base() -> Sudoku {
            let mut cells = [Cell {
                coords: Coords {
                    r: 0,
                    c: 0,
                    g: 0,
                    n: 0,
                },
                status: CellStatus::FIXED,
                candidates: Candidate::default(),
                value: None,
            }; 81];
            for r in 0..9 {
                for c in 0..9 {
                    let index = r * 9 + c;
                    let rc = RCCoords { r, c };
                    let value: Option<u8> = Some(((r * 3 + r / 3 + c) % 9) as u8);
                    cells[index].coords = rc.into();
                    if (r * 3) % 9 + r / 3 == c {
                        cells[index].candidates.add(value.unwrap());
                        cells[index].status = CellStatus::DRAFT;
                        // cells[index].value = None;
                    } else {
                        // cells[index].candidates No Need To Add;
                        cells[index].status = CellStatus::FIXED;
                        cells[index].value = value;
                    };
                }
            }
            Sudoku { cells }
        }

        fn dig_holes(sudoku: *mut Sudoku, max_digs: usize) {
            unsafe {
                let mut candidate_cells: Vec<*mut Cell> = (*sudoku)
                    .cells
                    .iter_mut()
                    .filter(|c| c.status == CellStatus::FIXED)
                    .map(|p| p as *mut Cell)
                    .collect();

                candidate_cells.shuffle(&mut rand::thread_rng());

                for cell in candidate_cells.into_iter().take(max_digs) {
                    let original = (*cell).value.take().unwrap();
                    (*cell).status = CellStatus::DRAFT;

                    // 更新相关候选数
                    update_peers_candidates(sudoku, cell, original);
                    // (*sudoku).print();
                    let mut bak = (*sudoku).clone();
                    let mut cnt: usize = 0;
                    if !is_unique(addr_of_mut!(bak), &mut cnt) {
                        (*cell).status = CellStatus::FIXED;
                        (*cell).value = Some(original);
                    }
                    // println!("{:?}", cnt)
                }
            }
        }

        unsafe fn is_unique(sudoku: *mut Sudoku, count: &mut usize) -> bool {
            // 假设格子的位置是r,c，那么这个格子的答案一定是(r * 3 + r / 3 + c) % 9)
            // 这里要想办法最快的推出矛盾来，假设挖空了这个格子，一定不能填其他数字
            // 从所有未填写的格子中，假设这个并非其意料之中的答案，排除候选数
            // 然后，观察是否可以得到矛盾——有个格子完全没有候选数，或者有个格子只能填写意料之外的数字
            unsafe fn is_valid(r: usize, c: usize, v: Option<u8>, field: *const Sudoku) -> bool {
                let GNCoords { g, n: _ } = RCCoords { r, c }.into();
                for i in 0..9 {
                    if (*field).get_cell_ref_by_rc(RCCoords { r, c: i }).value == v {
                        return false;
                    }
                    if (*field).get_cell_ref_by_rc(RCCoords { r: i, c }).value == v {
                        return false;
                    }
                    if (*field).get_cell_ref_by_gn(GNCoords { g, n: i }).value == v {
                        return false;
                    }
                }
                true
            }
            // 找到第一个草稿状态的单元格
            for r in 0..9 {
                for c in 0..9 {
                    let idx = r * 9 + c;
                    let cell: *mut Cell = addr_of_mut!((*sudoku).cells[idx]);
                    if (*cell).status == CellStatus::DRAFT {
                        for &num in &(*cell).candidates.to_vec() {
                            if is_valid(r, c, Some(num), sudoku) {
                                (*cell).value = Some(num);
                                (*cell).status = CellStatus::SOLVE;

                                if is_unique(sudoku, count) {
                                    *count += 1;
                                    if *count >= 2 {
                                        return false;
                                    };
                                }

                                // 回溯
                                (*cell).status = CellStatus::DRAFT;
                                (*cell).value = None; // 重置值
                            }
                        }
                        return false; // 如果没有找到有效的数字，则返回false
                    }
                }
            }

            true // 如果所有单元格都已解决，则返回true
        }

        unsafe fn update_peers_candidates(sudoku: *mut Sudoku, mid_cell: *const Cell, value: u8) {
            for i in 0..9 {
                // 更新行候选
                maybe_add_candidate(
                    sudoku,
                    (*sudoku).get_cell_mut_ptr_by_rc(RCCoords {
                        r: (*mid_cell).coords.r,
                        c: i,
                    }),
                    value,
                );
                // 更新列候选
                maybe_add_candidate(
                    sudoku,
                    (*sudoku).get_cell_mut_ptr_by_rc(RCCoords {
                        r: i,
                        c: (*mid_cell).coords.c,
                    }),
                    value,
                );
                // 更新宫候选
                maybe_add_candidate(
                    sudoku,
                    (*sudoku).get_cell_mut_ptr_by_gn(GNCoords {
                        g: (*mid_cell).coords.g,
                        n: i,
                    }),
                    value,
                );
            }
        }

        unsafe fn maybe_add_candidate(sudoku: *mut Sudoku, cell: *mut Cell, value: u8) {
            if ((*cell).coords.r == 0 && (*cell).coords.c == 0) {
                (*sudoku).print();
                println!(
                    "{:?},{:?},{:?}",
                    value,
                    (*cell).status,
                    conflict_exists(sudoku, cell, value)
                )
            }
            if (*cell).status == CellStatus::DRAFT && !conflict_exists(sudoku, cell, value) {
                (*cell).candidates.add(value);
            }
        }

        unsafe fn conflict_exists(sudoku: *mut Sudoku, cell: *const Cell, value: u8) -> bool {
            for i in 0..9 {
                let Cell {
                    status: cell_status,
                    value: cell_value,
                    ..
                } = (*sudoku).get_cell_ref_by_rc(RCCoords {
                    r: (*cell).coords.r,
                    c: i,
                });
                if *cell_status == CellStatus::FIXED && *cell_value == Some(value) {
                    return true;
                };
                let Cell {
                    status: cell_status,
                    value: cell_value,
                    ..
                } = (*sudoku).get_cell_ref_by_rc(RCCoords {
                    r: i,
                    c: (*cell).coords.c,
                });
                if *cell_status == CellStatus::FIXED && *cell_value == Some(value) {
                    return true;
                };
                let Cell {
                    status: cell_status,
                    value: cell_value,
                    ..
                } = (*sudoku).get_cell_ref_by_gn(GNCoords {
                    g: (*cell).coords.g,
                    n: i,
                });
                if *cell_status == CellStatus::FIXED && *cell_value == Some(value) {
                    return true;
                };
            }
            false
        }

        let mut sudoku = create_base();
        sudoku.print();
        // dig_holes(addr_of_mut!(sudoku), difficulty.empty_cells().1);

        // let mut rng = rand::thread_rng();

        // // 行交换
        // for _ in 0..3 {
        //     let block = rng.gen_range(0..3);
        //     let mut rows: Vec<usize> = (block * 3..(block + 1) * 3).collect();
        //     rows.shuffle(&mut rng);
        //     if let [r1, r2] = rows[..2] {
        //         puzzle.swap(r1, r2);
        //         solution.swap(r1, r2);
        //     }
        // }

        // // 列交换
        // for _ in 0..3 {
        //     let block = rng.gen_range(0..3);
        //     let mut cols: Vec<usize> = (block * 3..(block + 1) * 3).collect();
        //     cols.shuffle(&mut rng);
        //     if let [c1, c2] = cols[..2] {
        //         for row in puzzle.iter_mut() {
        //             row.swap(c1, c2);
        //         }
        //         for row in solution.iter_mut() {
        //             row.swap(c1, c2);
        //         }
        //     }
        // }

        // // 数字替换
        // let mut numbers: Vec<u8> = (1..=9).collect();
        // numbers.shuffle(&mut rng);
        // let replace_map: Vec<u8> = (1..=9).map(|i| numbers[i as usize - 1]).collect();

        // for i in 0..9 {
        //     for j in 0..9 {
        //         solution[i][j] = replace_map[solution[i][j] as usize - 1];
        //         if puzzle[i][j] != 0 {
        //             puzzle[i][j] = replace_map[puzzle[i][j] as usize - 1];
        //         }
        //     }
        // }

        //     field.assume_init()
        // };

        sudoku.fill_drafts();

        sudoku

        // let mut sudoku = Sudoku::create_base();

        // // 第一阶段：基础挖空
        // sudoku.dig_holes(27);  // 固定挖空27格

        // // 第二阶段：难度相关挖空
        // let extra_digs = match difficulty {
        //     Difficulty::Easy => 13,
        //     Difficulty::Medium => 18,
        //     Difficulty::Hard => 23,
        //     // ...
        // };
        // sudoku.dig_holes(extra_digs);

        // // 随机变换
        // sudoku.shuffle_rows_columns();
        // sudoku.remap_numbers();

        // sudoku
    }
}
