use std::{mem::MaybeUninit, ptr};

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct RCCoords {
    pub r: usize,
    pub c: usize,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct GNCoords {
    pub g: usize,
    pub n: usize,
}

impl RCCoords {
    pub fn to_gn_coords(&self) -> GNCoords {
        GNCoords {
            g: (self.r / 3 * 3 + self.c / 3),
            n: (self.r % 3 * 3 + self.c % 3),
        }
    }
}

impl GNCoords {
    pub fn to_rc_coords(&self) -> RCCoords {
        RCCoords {
            r: (self.g / 3 * 3 + self.n / 3),
            c: (self.g % 3 * 3 + self.n % 3),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Drafts {
    drafts: [bool; 9],
}

impl Drafts {
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

    pub fn find_value(&self, v: CellValue) -> Option<()> {
        if self.is_contain(v) {
            Some(())
        } else {
            None
        }
    }

    pub fn to_vec(&self) -> Vec<CellValue> {
        let mut ret = vec![];
        for i in 0..9 {
            if self.drafts[i] {
                ret.push(CellValue::from_value((i + 1) as u32).unwrap());
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

    pub fn vec_for_iter() -> Vec<CellValue> {
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
    }
}

#[derive(Debug, Clone)]
pub struct Cell {
    pub rc: RCCoords,
    pub gn: GNCoords,
    pub status: CellStatus,
    pub drafts: Drafts,
    pub value: CellValue,
}

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
        &mut self.cells[(gn.g / 3 * 3 + gn.n / 3) * 9 + (gn.g % 3 * 3 + gn.n % 3)]
    }

    pub fn get_cell_ref_by_gn(&self, gn: GNCoords) -> &Cell {
        &self.cells[(gn.g / 3 * 3 + gn.n / 3) * 9 + (gn.g % 3 * 3 + gn.n % 3)]
    }

    // 如果一个格子中没有任何候选数，说明中间过程出错了
    pub fn find_empty_drafts(&self) -> Option<Vec<&Cell>> {
        let mut ret: Vec<&Cell> = vec![];
        for p in &self.cells {
            if p.status == CellStatus::DRAFT {
                if !p.drafts.is_empty() {
                    ret.push(p);
                }
            }
        }
        if ret.len() > 0 {
            return Some(ret);
        }
        None
    }

    // 如果格子的内容有冲突，也说明有错误，可以不继续推理下去了
    pub fn find_conflict(&self) -> Option<Vec<(&Cell, &Cell)>> {
        let mut ret: Vec<(&Cell, &Cell)> = vec![];
        for r in 0..9 {
            for c in 0..9 {
                let rc = RCCoords { r, c };
                let tmp_rc = self.get_cell_ref_by_rc(rc);
                if tmp_rc.status == CellStatus::FIXED || tmp_rc.status == CellStatus::SOLVE {
                    let v = tmp_rc.value;
                    for r_iter in (r + 1)..9 {
                        let tmp = self.get_cell_ref_by_rc(RCCoords { r: r_iter, c: c });
                        if (tmp.value == v)
                            && (tmp.status == CellStatus::FIXED || tmp.status == CellStatus::SOLVE)
                        {
                            ret.push((tmp_rc, tmp));
                        }
                    }
                    for c_iter in (c + 1)..9 {
                        let tmp = self.get_cell_ref_by_rc(RCCoords { r: r, c: c_iter });
                        if (tmp.value == v)
                            && (tmp.status == CellStatus::FIXED || tmp.status == CellStatus::SOLVE)
                        {
                            ret.push((tmp_rc, tmp));
                        }
                    }
                    let gn = tmp_rc.gn;
                    for n_iter in (gn.n + 1)..9 {
                        let tmp = self.get_cell_ref_by_gn(GNCoords { g: gn.g, n: n_iter });
                        if (tmp.value == v)
                            && (tmp.status == CellStatus::FIXED || tmp.status == CellStatus::SOLVE)
                        {
                            ret.push((tmp_rc, tmp));
                        }
                    }
                }
            }
        }
        if ret.len() > 0 {
            return Some(ret);
        }
        None
    }

    // 仅在初始化时使用，补充所有可能的草稿数
    fn fill_drafts(&mut self) {
        for r in 0..9 {
            for c in 0..9 {
                let rc = RCCoords { r, c };
                let tmp_rc = self.get_cell_ref_by_rc(rc);
                if tmp_rc.status == CellStatus::FIXED || tmp_rc.status == CellStatus::SOLVE {
                    let v = self.get_cell_mut_by_rc(rc).value;
                    for r_iter in 0..9 {
                        self.get_cell_mut_by_rc(RCCoords { r: r_iter, c })
                            .drafts
                            .remove_draft(v);
                    }
                    for c_iter in 0..9 {
                        self.get_cell_mut_by_rc(RCCoords { r, c: c_iter })
                            .drafts
                            .remove_draft(v);
                    }
                    let gn = rc.to_gn_coords();
                    for n_iter in 0..9 {
                        let tmp_rc = GNCoords { g: gn.g, n: n_iter }.to_rc_coords();
                        self.get_cell_mut_by_rc(tmp_rc).drafts.remove_draft(v)
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
            let mut field = MaybeUninit::<Field>::uninit();
            let p_field: *mut Field = field.as_mut_ptr();
            let p_cell: *mut Cell = (*p_field).cells.as_mut_ptr();

            for (index, item) in input.chars().enumerate() {
                let tmp = item.to_digit(10).expect("Invalid Character.");
                let rc = RCCoords {
                    r: index / 9,
                    c: index % 9,
                };
                let gn = rc.to_gn_coords();
                if tmp == 0 {
                    ptr::write(
                        p_cell.offset(index as isize),
                        Cell {
                            rc: rc,
                            gn: gn,
                            status: CellStatus::DRAFT,
                            drafts: Drafts { drafts: [true; 9] },
                            value: CellValue::INVAILD,
                        },
                    );
                } else {
                    ptr::write(
                        p_cell.offset(index as isize),
                        Cell {
                            rc: rc,
                            gn: gn,
                            status: CellStatus::FIXED,
                            drafts: Drafts { drafts: [true; 9] },
                            value: CellValue::from_value(tmp)?,
                        },
                    );
                }
            }

            field.assume_init()
        };

        field.fill_drafts();

        Ok(field)
    }

    // 打印数独，用特殊效果显示草稿、固定值、填写值
    pub fn print(&self) {
        println!("╔═══╤═══╤═══╦═══╤═══╤═══╦═══╤═══╤═══╗");
        for r in 0..9 {
            for m in 0..3 {
                let mut line: String = "║".to_string();
                for c in 0..9 {
                    let p = &self.cells[r * 9 + c];
                    if p.status == CellStatus::DRAFT {
                        for n in 0..3 {
                            let d = m * 3 + n;
                            if p.drafts.drafts[d] {
                                line += &((d + 1).to_string());
                            } else {
                                line += " ";
                            }
                        }
                    } else if p.status == CellStatus::FIXED {
                        if m == 0 {
                            line += "\\ /";
                        } else if m == 1 {
                            line += " ";
                            line += &((p.value as u32).to_string());
                            line += " ";
                        } else {
                            line += "/ \\";
                        }
                    } else if p.status == CellStatus::SOLVE {
                        if m == 0 {
                            line += "***";
                        } else if m == 1 {
                            line += "*";
                            line += &((p.value as u32).to_string());
                            line += "*";
                        } else {
                            line += "***";
                        }
                    }
                    if c % 3 == 2 {
                        line += "║";
                    } else {
                        line += "│";
                    }
                }
                println!("{}", line);
            }
            if r == 8 {
                println!("╚═══╧═══╧═══╩═══╧═══╧═══╩═══╧═══╧═══╝")
            } else if r % 3 == 2 {
                println!("╠═══╪═══╪═══╬═══╪═══╪═══╬═══╪═══╪═══╣")
            } else {
                println!("╟───┼───┼───╫───┼───┼───╫───┼───┼───╢")
            }
        }
    }

    // 以下是常见的遍历手段

    // 遍历所有单元格
    pub fn collect_all_drafts_cells(&self) -> Vec<&Cell> {
        self.cells
            .iter()
            .filter(|&p| p.status == CellStatus::DRAFT)
            .collect::<Vec<&Cell>>()
    }

    // 按行遍历草稿单元格
    pub fn collect_all_drafts_cells_by_rc(&self) -> Vec<Vec<&Cell>> {
        (0..9)
            .into_iter()
            .map(|r| {
                (0..9)
                    .into_iter()
                    .map(|c| self.get_cell_ref_by_rc(RCCoords { r, c }))
                    .filter(|&p| (*p).status == CellStatus::DRAFT)
                    .collect()
            })
            .collect()
    }

    // 按列遍历草稿单元格
    pub fn collect_all_drafts_cells_by_cr(&self) -> Vec<Vec<&Cell>> {
        (0..9)
            .into_iter()
            .map(|c| {
                (0..9)
                    .into_iter()
                    .map(|r| self.get_cell_ref_by_rc(RCCoords { r, c }))
                    .filter(|&p| (*p).status == CellStatus::DRAFT)
                    .collect()
            })
            .collect()
    }

    // 按宫遍历草稿单元格
    pub fn collect_all_drafts_cells_by_gn(&self) -> Vec<Vec<&Cell>> {
        (0..9)
            .into_iter()
            .map(|g| {
                (0..9)
                    .into_iter()
                    .map(|n| self.get_cell_ref_by_gn(GNCoords { g, n }))
                    .filter(|&p| (*p).status == CellStatus::DRAFT)
                    .collect()
            })
            .collect()
    }

    // 在指定行按列遍历单元格
    pub fn collect_all_drafts_cells_in_r(&self, r: usize) -> Vec<&Cell> {
        (0..9)
            .into_iter()
            .map(|c| self.get_cell_ref_by_rc(RCCoords { r, c }))
            .filter(|&p| (*p).status == CellStatus::DRAFT)
            .collect()
    }

    // 在指定列按行遍历单元格
    pub fn collect_all_drafts_cells_in_c(&self, c: usize) -> Vec<&Cell> {
        (0..9)
            .into_iter()
            .map(|r| self.get_cell_ref_by_rc(RCCoords { r, c }))
            .filter(|&p| (*p).status == CellStatus::DRAFT)
            .collect()
    }

    // 在指定宫按序遍历单元格
    pub fn collect_all_drafts_cells_in_g(&self, g: usize) -> Vec<&Cell> {
        (0..9)
            .into_iter()
            .map(|n| self.get_cell_ref_by_gn(GNCoords { g, n }))
            .filter(|&p| (*p).status == CellStatus::DRAFT)
            .collect()
    }

    // 检查是否都填写完毕了
    pub fn check_if_finish(&self) -> bool {
        self.collect_all_drafts_cells().len() == 0
    }
}
