use std::fmt::write;

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

#[derive(Copy, Clone, PartialEq)]
/// 兼容上述两种坐标系
pub enum Coords {
    RC(RCCoords),
    GN(GNCoords),
}

impl Coords {
    pub fn to_rc_coords(&self) -> RCCoords {
        match self {
            Coords::RC(ret) => *ret,
            Coords::GN(gn) => gn.to_rc_coords(),
        }
    }

    pub fn to_gn_coords(&self) -> GNCoords {
        match self {
            Coords::RC(rc) => rc.to_gn_coords(),
            Coords::GN(ret) => *ret,
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
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

    pub fn find_value(&self, v: CellValue) -> Option<CellValue> {
        self.is_contain(v).then_some(v)
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

    pub fn delta_to(&self, other: Drafts) -> usize {
        let mut ret = 0;
        (0..9).for_each(|i| {
            if self.drafts[i] != other.drafts[i] {
                ret += 1;
            }
        });
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
        ].into_iter()
    }
}

#[derive(Clone)]
pub struct Cell {
    pub rc: RCCoords,
    pub gn: GNCoords,
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

impl std::fmt::Debug for Coords {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Coords::RC(RCCoords { r, c }) => {
                write!(f, "R{}C{}", r + 1, c + 1)
            }
            Coords::GN(GNCoords { g, n }) => {
                write!(f, "G{}N{}", g + 1, n + 1)
            }
        }
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

/// 某某策略的结论通常可以归纳为：因为【某个地方的某个值】，导致【某个地方的某个值】，需要做一些什么
/// 这里定义的是【某个地方的某个值】
#[derive(Clone)]
pub struct TheCellAndTheValue<'a> {
    pub the_cell: &'a Cell,
    pub the_value: CellValue,
}

impl std::fmt::Debug for TheCellAndTheValue<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}V{:?}", self.the_cell.rc, self.the_value)
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
        &mut self.cells[(gn.g / 3 * 3 + gn.n / 3) * 9 + (gn.g % 3 * 3 + gn.n % 3)]
    }

    pub fn get_cell_ref_by_gn(&self, gn: GNCoords) -> &Cell {
        &self.cells[(gn.g / 3 * 3 + gn.n / 3) * 9 + (gn.g % 3 * 3 + gn.n % 3)]
    }

    pub fn get_cell_mut_by_coords(&mut self, coords: Coords) -> &mut Cell {
        match coords {
            Coords::RC(rc) => self.get_cell_mut_by_rc(rc),
            Coords::GN(gn) => self.get_cell_mut_by_gn(gn),
        }
    }

    pub fn get_cell_ref_by_coords(&self, coords: Coords) -> &Cell {
        match coords {
            Coords::RC(rc) => self.get_cell_ref_by_rc(rc),
            Coords::GN(gn) => self.get_cell_ref_by_gn(gn),
        }
    }

    pub fn set_cell_value(&mut self, ref input: Cell) {
        let p = &mut (self.cells[input.rc.r * 9 + input.rc.c]);
        p.status = input.status;
        p.value = input.value;
        p.drafts = input.drafts;
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
                    let v = self.get_cell_ref_by_rc(rc).value;
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
            let mut field = std::mem::MaybeUninit::<Field>::uninit();
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
                    std::ptr::write(
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
                    std::ptr::write(
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

    /// 给定一个坐标，根据坐标遍历同一行、同一列、同一宫所有可辐射的单元格（若自身为可操作单元格，则包括自身）
    pub fn collect_all_drafts_cells_by_coords(&self, coords: Coords) -> Vec<&Cell> {
        let RCCoords { r, c } = coords.to_rc_coords();
        let GNCoords { g, n: _ } = coords.to_gn_coords();

        // 先搜集行
        (0..9)
            .into_iter()
            .map(|c_iter| self.get_cell_ref_by_rc(RCCoords { r, c: c_iter }))
            .filter(|&p| (*p).status == CellStatus::DRAFT)
            .chain(
                // 再收集列，注意去重
                (0..9)
                    .into_iter()
                    .map(|r_iter| self.get_cell_ref_by_rc(RCCoords { r: r_iter, c }))
                    .filter(|&p| (*p).status == CellStatus::DRAFT && (*p).rc.r != r),
            )
            .chain(
                (0..9)
                    .into_iter()
                    .map(|n_iter| self.get_cell_ref_by_gn(GNCoords { g, n: n_iter }))
                    .filter(|&p| {
                        (*p).status == CellStatus::DRAFT && (*p).rc.r != r && (*p).rc.c != c
                    }),
            )
            .collect()
    }

    // 当某个格子设置某个值的时候，将同行列宫的该值的草稿值移除，输入值在vec_set_value.cells内，且value唯一
    pub fn collect_all_drafts_coords_by_the_coords_and_the_value(
        &self,
        middle_cell: &Cell,
        value: CellValue,
    ) -> Option<Vec<TheCellAndTheValue>> {
        // let RCCoords { r, c } = middle_cell.rc;

        let ret: Vec<TheCellAndTheValue> = self
            .collect_all_drafts_cells_by_coords(Coords::RC(middle_cell.rc.clone()))
            .iter()
            .filter_map(|&p| {
                if p.drafts.is_contain(value) {
                    Some(TheCellAndTheValue {
                        the_cell: *&p,
                        the_value: value,
                    })
                } else {
                    None
                }
            })
            .collect();

        if ret.len() != 0 {
            Some(ret)
        } else {
            None
        }
    }

    /// 检查是否都填写完毕了
    pub fn check_if_finish(&self) -> bool {
        self.collect_all_drafts_cells().len() == 0
    }
}
