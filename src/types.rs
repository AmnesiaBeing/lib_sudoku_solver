use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::utils::create_simple_cell_and_value;

#[derive(Copy, Clone, PartialEq)]
pub struct RCCoords {
    pub r: usize,
    pub c: usize,
}

#[derive(Copy, Clone, PartialEq)]
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

#[derive(Default, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Drafts {
    pub drafts: [bool; 9],
}

impl Drafts {
    pub fn new_all_false() -> Drafts {
        Drafts { drafts: [false; 9] }
    }

    pub fn new_all_true() -> Drafts {
        Drafts { drafts: [true; 9] }
    }

    pub fn is_empty(&self) -> bool {
        self.drafts.contains(&true)
    }

    pub fn try_get_the_only_one(&self) -> Option<CellValue> {
        let mut flag = false;
        let mut ret = 0;
        for i in 0..9 {
            if self.drafts[i] {
                if flag {
                    return None;
                }
                ret = i;
                flag = true;
            }
        }
        return Some(CellValue::from_value((ret + 1) as u32).unwrap());
    }

    pub fn add_draft(&mut self, v: CellValue) {
        if v != CellValue::INVAILD {
            self.drafts[v.to_index().unwrap()] = true;
        }
    }

    pub fn remove_draft(&mut self, v: CellValue) {
        if v != CellValue::INVAILD {
            self.drafts[v.to_index().unwrap()] = false;
        }
    }

    pub fn is_contain(&self, v: CellValue) -> bool {
        self.drafts[v.to_index().unwrap()]
    }

    pub fn find_value(&self, v: CellValue) -> Option<CellValue> {
        self.is_contain(v).then_some(v)
    }

    pub fn to_vec(&self) -> Vec<CellValue> {
        self.drafts
            .iter()
            .enumerate()
            .filter_map(|(i, &draft)| {
                if draft {
                    Some(CellValue::from_value((i + 1) as u32).unwrap())
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn delta_to(&self, other: Drafts) -> usize {
        self.drafts
            .iter()
            .zip(other.drafts.iter())
            .filter(|(a, b)| a != b)
            .count()
    }

    pub fn len(&self) -> usize {
        self.drafts.iter().filter(|&&draft| draft).count()
    }

    pub fn union(&self, other: Drafts) -> Drafts {
        let mut ret: Drafts = Drafts::default();
        for i in 0..9 {
            if self.drafts[i] || other.drafts[i] {
                ret.drafts[i] = true;
            }
        }
        ret
    }

    pub fn intersect(&self, other: Drafts) -> Drafts {
        let mut ret: Drafts = Drafts::default();
        for i in 0..9 {
            if self.drafts[i] && other.drafts[i] {
                ret.drafts[i] = true;
            }
        }
        ret
    }

    pub fn subtract(&self, other: Drafts) -> Drafts {
        let mut ret: Drafts = Drafts::default();
        for i in 0..9 {
            if self.drafts[i] && !other.drafts[i] {
                ret.drafts[i] = true;
            }
        }
        ret
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum CellStatus {
    // 固定数值
    FIXED,
    // 草稿，未填值
    DRAFT,
    // 用户的解答，已填值，此时drafts数组的内容将被忽略
    SOLVE,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum CellValue {
    INVAILD = 0,
    V1 = 1,
    V2 = 2,
    V3 = 3,
    V4 = 4,
    V5 = 5,
    V6 = 6,
    V7 = 7,
    V8 = 8,
    V9 = 9,
}

impl CellValue {
    pub fn from_value(v: u32) -> Result<CellValue, &'static str> {
        match v {
            0 => Ok(CellValue::INVAILD),
            1 => Ok(CellValue::V1),
            2 => Ok(CellValue::V2),
            3 => Ok(CellValue::V3),
            4 => Ok(CellValue::V4),
            5 => Ok(CellValue::V5),
            6 => Ok(CellValue::V6),
            7 => Ok(CellValue::V7),
            8 => Ok(CellValue::V8),
            9 => Ok(CellValue::V9),
            _ => Err("Invalid Cell Value."),
        }
    }

    pub fn to_index(&self) -> Result<usize, &'static str> {
        match *self {
            CellValue::V1 => Ok(0),
            CellValue::V2 => Ok(1),
            CellValue::V3 => Ok(2),
            CellValue::V4 => Ok(3),
            CellValue::V5 => Ok(4),
            CellValue::V6 => Ok(5),
            CellValue::V7 => Ok(6),
            CellValue::V8 => Ok(7),
            CellValue::V9 => Ok(8),
            CellValue::INVAILD => Err("Invalid Cell Value."),
        }
    }

    pub fn iter() -> <Vec<CellValue> as IntoIterator>::IntoIter {
        vec![
            CellValue::V1,
            CellValue::V2,
            CellValue::V3,
            CellValue::V4,
            CellValue::V5,
            CellValue::V6,
            CellValue::V7,
            CellValue::V8,
            CellValue::V9,
        ]
        .into_iter()
    }
}

#[derive(Clone)]
pub struct Cell {
    pub rc: RCCoords,
    pub gn: GNCoords,
    pub coords: Coords,
    pub status: CellStatus,
    pub drafts: Drafts,
    pub value: CellValue,
}

impl std::fmt::Debug for Drafts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for i in 0..9 {
            if self.drafts[i] {
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

impl std::fmt::Debug for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}{:?}", self.rc, self.gn,)?;
        match self.status {
            CellStatus::FIXED | CellStatus::SOLVE => write!(f, "{:?};", self.value),
            CellStatus::DRAFT => {
                write!(f, "D{:?};", self.drafts)
            }
        }
    }
}

/// 数组本体
#[derive(Clone)]
pub struct Field {
    cells: [Cell; 81],
}

impl Field {
    pub fn get_cell_mut_by_rc(&mut self, rc: RCCoords) -> &mut Cell {
        &mut self.cells[rc.r * 9 + rc.c]
    }

    pub fn get_cell_ref_by_rc(&self, rc: RCCoords) -> &Cell {
        &self.cells[rc.r * 9 + rc.c]
    }

    pub fn get_cell_mut_by_gn(&mut self, gn: GNCoords) -> &mut Cell {
        let RCCoords { r, c } = gn.into();
        &mut self.cells[r * 9 + c]
    }

    pub fn get_cell_ref_by_gn(&self, gn: GNCoords) -> &Cell {
        let RCCoords { r, c } = gn.into();
        &self.cells[r * 9 + c]
    }

    pub fn get_cell_mut_by_coords(&mut self, coords: Coords) -> &mut Cell {
        let Coords { r, c, g: _, n: _ } = coords;
        &mut self.cells[r * 9 + c]
    }

    pub fn get_cell_ref_by_coords(&self, coords: Coords) -> &Cell {
        let Coords { r, c, g: _, n: _ } = coords;
        &self.cells[r * 9 + c]
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
                    let g = p_cell.gn.g;
                    let n = p_cell.gn.n;
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
                let rc = RCCoords { r, c };
                let &Cell {
                    status,
                    coords: _,
                    value,
                    rc: _,
                    gn,
                    drafts: _,
                } = self.get_cell_ref_by_rc(rc);
                if status == CellStatus::FIXED {
                    for i in 0..9 {
                        let p_cell = self.get_cell_mut_by_rc(RCCoords { r: i, c });
                        p_cell.drafts.remove_draft(value);

                        let p_cell = self.get_cell_mut_by_rc(RCCoords { r, c: i });
                        p_cell.drafts.remove_draft(value);

                        let g = gn.g;
                        let p_cell = self.get_cell_mut_by_gn(GNCoords { g, n: i });
                        p_cell.drafts.remove_draft(value);
                    }
                }
            }
        }
    }

    // 从字符串初始化数独，要求输入字符串长度必须为81，且仅为0-9的数字
    pub fn initial_by_string(input: &String) -> Result<Field, &'static str> {
        if input.len() != 81 {
            return Err("Invalid String Length.");
        }

        let mut field: Field = unsafe {
            let mut field = std::mem::MaybeUninit::<Field>::uninit();
            let p_field: *mut Field = field.as_mut_ptr();
            let p_cell: *mut Cell = (*p_field).cells.as_mut_ptr();

            for (index, item) in input.chars().enumerate() {
                let tmp = item.to_digit(10).expect("Invalid Character.");
                let rc = RCCoords::from_idx(index);
                let gn = rc.into();
                let coords = rc.into();
                let status = if tmp == 0 {
                    CellStatus::DRAFT
                } else {
                    CellStatus::FIXED
                };
                let value = if tmp == 0 {
                    CellValue::INVAILD
                } else {
                    CellValue::from_value(tmp).expect("Invalid Value.")
                };
                std::ptr::write(
                    p_cell.offset(index as isize),
                    Cell {
                        rc,
                        gn,
                        coords,
                        status,
                        drafts: Drafts::new_all_true(),
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
    pub fn initial_by_random() {
        unsafe fn swap_row(field: *mut Field, r1: usize, r2: usize) {
            (0..9)
                .into_iter()
                .for_each(|c| (*field).cells.swap(r1 * 9 + c, r2 * 9 + c));
        }
        unsafe fn swap_col(field: *mut Field, c1: usize, c2: usize) {
            (0..9)
                .into_iter()
                .for_each(|r| (*field).cells.swap(r * 9 + c1, r * 9 + c2));
        }

        let mut field = Field::initial_by_string(
            &"123456789456789123789123456891234567234567891567891234678912345912345678345678912"
                .to_string(),
        )
        .unwrap();

        let mut rng = rand::thread_rng();

        loop {
            for _ in 0..2000 {
                let a = rng.gen_range(0..3);
                let b = rng.gen_range(0..3);
                let c = rng.gen_range(0..3);

                match rng.gen_bool(0.5) {
                    true => unsafe {
                        swap_row(core::ptr::addr_of_mut!(field), c * 3 + a, c * 3 + b)
                    },
                    false => unsafe {
                        swap_col(core::ptr::addr_of_mut!(field), c * 3 + a, c * 3 + b)
                    },
                }
            }

            for _ in 0..(81 - 25) {
                let idx = rng.gen_range(0..81);
                field.cells[idx].status = CellStatus::DRAFT;
                field.cells[idx].value = CellValue::INVAILD;
            }

            field.fill_drafts();

            if field.sovle().len() == 1 {
                break;
            }
        }

        field.print();
    }

    // 打印数独，用特殊效果显示草稿、固定值、填写值
    pub fn print(&self) {
        println!("╔═══╤═══╤═══╦═══╤═══╤═══╦═══╤═══╤═══╗");
        for r in 0..9 {
            for m in 0..3 {
                let mut line = String::from("║");
                for c in 0..9 {
                    let p = &self.cells[r * 9 + c];
                    line += &match p.status {
                        CellStatus::DRAFT => (0..3)
                            .map(|n| {
                                if p.drafts.drafts[m * 3 + n] {
                                    (m * 3 + n + 1).to_string()
                                } else {
                                    " ".to_string()
                                }
                            })
                            .collect::<String>(),
                        CellStatus::FIXED => match m {
                            0 => "\\ /".to_string(),
                            1 => format!(" {} ", p.value as u32),
                            _ => "/ \\".to_string(),
                        },
                        CellStatus::SOLVE => match m {
                            0 => "***".to_string(),
                            1 => format!("*{}*", p.value as u32),
                            _ => "***".to_string(),
                        },
                    };
                    line += if c % 3 == 2 { "║" } else { "│" };
                }
                println!("{}", line);
            }
            if r == 8 {
                println!("╚═══╧═══╧═══╩═══╧═══╧═══╩═══╧═══╧═══╝");
            } else if r % 3 == 2 {
                println!("╠═══╪═══╪═══╬═══╪═══╪═══╬═══╪═══╪═══╣");
            } else {
                println!("╟───┼───┼───╫───┼───┼───╫───┼───┼───╢");
            }
        }
    }

    pub fn sovle(&self) -> Vec<Field> {
        let mut field = self.clone();
        let mut solutions: Vec<Field> = Vec::new();

        unsafe fn self_solve_field(field: &mut Field, solutions: &mut Vec<Field>) -> bool {
            fn is_valid(r: usize, c: usize, v: CellValue, field: &Field) -> bool {
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

            // 找到第一个草稿状态的单元格
            for r in 0..9 {
                for c in 0..9 {
                    let idx = r * 9 + c;
                    let cell: *mut Cell = core::ptr::addr_of_mut!(field.cells[idx]);
                    if (*cell).status == CellStatus::DRAFT {
                        for &num in &(*cell).drafts.to_vec() {
                            if is_valid(r, c, num, &field) {
                                (*cell).value = num;
                                (*cell).status = CellStatus::SOLVE;

                                if self_solve_field(field, solutions) {
                                    solutions.push(field.clone());
                                    if solutions.len() >= 2 {
                                        return true;
                                    }
                                }

                                // 回溯
                                (*cell).status = CellStatus::DRAFT;
                                (*cell).value = CellValue::INVAILD; // 重置值
                            }
                        }
                        return false; // 如果没有找到有效的数字，则返回false
                    }
                }
            }

            true // 如果所有单元格都已解决，则返回true
        }

        unsafe {
            self_solve_field(&mut field, &mut solutions);
        }

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
    pub fn iter_all_drafts_cells_by_rc_contain_v(
        &self,
        v: CellValue,
    ) -> <Vec<Vec<&Cell>> as IntoIterator>::IntoIter {
        (0..9)
            .into_iter()
            .map(|r| {
                (0..9)
                    .into_iter()
                    .map(|c| self.get_cell_ref_by_rc(RCCoords { r, c }))
                    .filter(|&p| (*p).status == CellStatus::DRAFT && (*p).drafts.is_contain(v))
                    .collect()
            })
            .collect::<Vec<Vec<&Cell>>>()
            .into_iter()
    }

    /// 给定一个坐标和值，根据坐标遍历同一行、同一列、同一宫所有含有这个值的单元格
    pub fn collect_all_drafts_coords_by_coords_and_value(
        &self,
        coords: Coords,
        value: CellValue,
    ) -> Vec<Coords> {
        let Coords { r, c, g, n: _ } = coords;

        let mut coords = vec![];

        for i in 0..9 {
            let p_cell = self.get_cell_ref_by_rc(RCCoords { r: i, c });
            if p_cell.status == CellStatus::DRAFT && p_cell.drafts.is_contain(value) {
                coords.push(p_cell.coords);
            }

            let p_cell = self.get_cell_ref_by_rc(RCCoords { r, c: i });
            if p_cell.rc.c != c
                && p_cell.status == CellStatus::DRAFT
                && p_cell.drafts.is_contain(value)
            {
                coords.push(p_cell.coords);
            }

            let p_cell = self.get_cell_ref_by_gn(GNCoords { g, n: i });
            if p_cell.rc.r != r
                && p_cell.rc.c != c
                && p_cell.status == CellStatus::DRAFT
                && p_cell.drafts.is_contain(value)
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
