use core::fmt;
use std::{collections::btree_set::Union, fmt::write, mem::MaybeUninit, ptr};

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct RCCoords {
    r: usize,
    c: usize,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct GNCoords {
    g: usize,
    n: usize,
}

pub enum CoordsType {
    RC,
    GN,
}

union _Coords {
    rc: RCCoords,
    gc: RCCoords,
}

pub struct Coords {
    coords: _Coords,
    coords_type: CoordsType,
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
struct Drafts {
    drafts: [bool; 9],
}

impl Drafts {
    pub fn is_empty(&self) -> bool {
        self.drafts.contains(&true)
    }

    pub fn try_get_the_only_one(&self) -> Option<CellValue> {
        let mut cnt = 0;
        let mut ret = 0;
        for i in 0..9 {
            if self.drafts[i] {
                cnt += 1;
                ret = i;
            }
        }
        if cnt == 1 {
            return Some(CellValue::from_value(ret as u32).unwrap());
        } else {
            return None;
        }
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
}

#[derive(Debug, Clone)]
pub struct Cell {
    rc: RCCoords,
    gn: GNCoords,
    status: CellStatus,
    drafts: Drafts,
    value: CellValue,
}

#[derive(Clone)]
pub struct Field {
    cells: [Cell; 81],
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Situation {
    SetValue,
    RemoveDrafts,
    OnlyOneLeft,
    OnlyOneRightInRow,
    OnlyOneRightInCol,
    OnlyOneInGrid,
}

pub struct Operator<'a> {
    situation: Situation,
    cell: &'a Cell,
    value: Option<CellValue>,
    drafts: Option<Drafts>,
}

type FnInference = fn(&Field) -> Option<Inference>;

pub struct Inference<'a> {
    condition: Vec<Operator<'a>>,
    conclusion: Vec<Operator<'a>>,
}

impl fmt::Display for Inference<'_> {
    /// # 实现调试接口
    /// R行
    /// C列
    /// G宫
    /// N宫内序号
    /// D草稿
    /// V值
    /// -去除
    /// +设置值
    /// &条件与、结论与
    /// ^因为
    /// =推导
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.condition.len() != 0 {
            write!(f, "^");
            for (index, c) in self.condition.iter().enumerate() {
                write!(f, "R{}C{}", c.cell.rc.r, c.cell.rc.c)?;
                if index < self.condition.len() {
                    write!(f, "&")?;
                }
            }
        }
        if self.conclusion.len() != 0 {
            write!(f, "=")?;
            for (index, c) in self.conclusion.iter().enumerate() {
                write!(f, "R{}C{}", c.cell.rc.r, c.cell.rc.c)?;
                if index < self.conclusion.len() {
                    write!(f, "&")?;
                }
            }
        }
        write!(f, "")
    }
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

    fn make_conclusion_when_set_value<'a>(
        field: &'a Field,
        set_value_operator: Operator<'a>,
    ) -> Vec<Operator<'a>> {
        let mut ret: Vec<Operator> = vec![];
        let v = set_value_operator.value.unwrap();

        for r_iter in 0..9 {
            if r_iter != set_value_operator.cell.rc.r {
                let p = field.get_cell_ref_by_rc(RCCoords {
                    r: r_iter,
                    c: set_value_operator.cell.rc.c,
                });
                if p.status == CellStatus::DRAFT {
                    if p.drafts.is_contain(v) {
                        // let mut d = Drafts { drafts: [false; 9] };
                        // d.remove_draft(set_value_operator.value.unwrap());
                        ret.push(Operator {
                            situation: Situation::RemoveDrafts,
                            cell: p,
                            value: Some(v),
                            drafts: None,
                        })
                    }
                }
            }
        }

        for c_iter in 0..9 {
            if c_iter != set_value_operator.cell.rc.c {
                let p = field.get_cell_ref_by_rc(RCCoords {
                    r: set_value_operator.cell.rc.r,
                    c: c_iter,
                });
                if p.status == CellStatus::DRAFT {
                    if p.drafts.is_contain(v) {
                        // let mut d = Drafts { drafts: [false; 9] };
                        // d.remove_draft(set_value_operator.value.unwrap());
                        ret.push(Operator {
                            situation: Situation::RemoveDrafts,
                            cell: p,
                            value: Some(v),
                            drafts: None,
                        })
                    }
                }
            }
        }

        for n_iter in 0..9 {
            if n_iter != set_value_operator.cell.gn.n {
                let p = field.get_cell_ref_by_gn(GNCoords {
                    g: set_value_operator.cell.gn.g,
                    n: n_iter,
                });
                if p.status == CellStatus::DRAFT {
                    if p.drafts.is_contain(v) {
                        // let mut d = Drafts { drafts: [false; 9] };
                        // d.remove_draft(set_value_operator.value.unwrap());
                        ret.push(Operator {
                            situation: Situation::RemoveDrafts,
                            cell: p,
                            value: Some(v),
                            drafts: None,
                        })
                    }
                }
            }
        }

        ret.push(set_value_operator);

        ret
    }

    // 唯余法
    pub fn inference_only_one_left(&self) -> Option<Inference> {
        let mut ret = Inference {
            condition: vec![],
            conclusion: vec![],
        };
        for p in &self.cells {
            if p.status == CellStatus::DRAFT {
                // 对于每一个处于草稿状态的格子，都尝试判断可填写草稿数是否为1
                let opt = p.drafts.try_get_the_only_one();
                if opt.is_none() {
                    continue;
                }
                // 如果是，则满足“唯余法”的条件
                ret.condition.push(Operator {
                    situation: Situation::OnlyOneLeft,
                    cell: p,
                    value: None,
                    drafts: Some(p.drafts),
                });
                // 同时，该格子需要填写该数字
                let sv_op = Operator {
                    situation: Situation::SetValue,
                    cell: p,
                    value: Some(opt.unwrap()),
                    drafts: None,
                };
                // 总的结果是，该格子需要填写该数字，同时，同一行列宫内的该数字都需要去除
                ret.conclusion
                    .append(&mut Self::make_conclusion_when_set_value(self, sv_op));
            }
        }
        if ret.condition.len() != 0 && ret.conclusion.len() != 0 {
            return Some(ret);
        } else {
            return None;
        }
    }

    // 排除法，对于每个格子内的草稿值，按照每行、每列、每宫方向进行判断，如果唯一，则填写该值，同时去除其余同一行列宫的草稿值
    pub fn inference_only_one_right(&self) -> Option<Inference> {
        let mut ret = Inference {
            condition: vec![],
            conclusion: vec![],
        };
        for r in 0..9 {
            for c in 0..9 {
                let p = self.get_cell_ref_by_rc(RCCoords { r: r, c: c });
                if p.status == CellStatus::DRAFT {
                    for (_, d) in p.drafts.to_vec().iter().enumerate() {
                        // 按列检索
                        {
                            let mut flag = true;
                            for r_iter in 0..9 {
                                if r_iter == r {
                                    continue;
                                }
                                let p_iter = self.get_cell_ref_by_rc(RCCoords { r: r_iter, c: c });
                                if p_iter.status == CellStatus::DRAFT {
                                    if p_iter.drafts.is_contain(*d) {
                                        flag = false;
                                        break;
                                    }
                                }
                            }
                            if flag {
                                // 该草稿唯一
                                ret.condition.push(Operator {
                                    situation: Situation::OnlyOneRightInCol,
                                    cell: p,
                                    value: Some(*d),
                                    drafts: None,
                                });
                                // 同时，该格子需要填写该数字
                                let sv_op = Operator {
                                    situation: Situation::SetValue,
                                    cell: p,
                                    value: Some(*d),
                                    drafts: None,
                                };
                                // 总的结果是，该格子需要填写该数字，同时，同一行列宫内的该数字都需要去除
                                ret.conclusion
                                    .append(&mut Self::make_conclusion_when_set_value(self, sv_op));
                                return Some(ret);
                            }
                        }
                        // 按行检索
                        {
                            let mut flag = true;
                            for c_iter in 0..9 {
                                if c_iter == c {
                                    continue;
                                }
                                let p_iter = self.get_cell_ref_by_rc(RCCoords { r: r, c: c_iter });
                                if p_iter.status == CellStatus::DRAFT {
                                    if p_iter.drafts.is_contain(*d) {
                                        flag = false;
                                        break;
                                    }
                                }
                            }
                            if flag {
                                // 该草稿唯一
                                ret.condition.push(Operator {
                                    situation: Situation::OnlyOneRightInCol,
                                    cell: p,
                                    value: Some(*d),
                                    drafts: None,
                                });
                                // 同时，该格子需要填写该数字
                                let sv_op = Operator {
                                    situation: Situation::SetValue,
                                    cell: p,
                                    value: Some(*d),
                                    drafts: None,
                                };
                                // 总的结果是，该格子需要填写该数字，同时，同一行列宫内的该数字都需要去除
                                ret.conclusion
                                    .append(&mut Self::make_conclusion_when_set_value(self, sv_op));
                                return Some(ret);
                            }
                        }
                        // 按宫检索
                        {
                            let mut flag = true;
                            for n_iter in 0..9 {
                                if n_iter == p.gn.n {
                                    continue;
                                }
                                let p_iter = self.get_cell_ref_by_gn(GNCoords {
                                    g: p.gn.g,
                                    n: n_iter,
                                });
                                if p_iter.status == CellStatus::DRAFT {
                                    if p_iter.drafts.is_contain(*d) {
                                        flag = false;
                                        break;
                                    }
                                }
                            }
                            if flag {
                                // 该草稿唯一
                                ret.condition.push(Operator {
                                    situation: Situation::OnlyOneRightInCol,
                                    cell: p,
                                    value: Some(*d),
                                    drafts: None,
                                });
                                // 同时，该格子需要填写该数字
                                let sv_op = Operator {
                                    situation: Situation::SetValue,
                                    cell: p,
                                    value: Some(*d),
                                    drafts: None,
                                };
                                // 总的结果是，该格子需要填写该数字，同时，同一行列宫内的该数字都需要去除
                                ret.conclusion
                                    .append(&mut Self::make_conclusion_when_set_value(self, sv_op));
                                return Some(ret);
                            }
                        }
                    }
                }
            }
        }
        None
    }

    // 高级排除法
    // pub fn inference_only_one_right_ex(&self) -> Option<Inference> {}

    // 数对排除法
    // pub fn inference_cell_pair(&self) -> Option<Inference> {}

    /// # 数独推理过程
    /// 1. 对每个格子判断唯一性，当只有1个候选数时，该格子必定为此数，填写该数，同时去除同一行列宫的该草稿数（唯余法）
    /// 2. 对行列宫判断唯一性，当一个候选数在同一行列宫只有唯一选项时，填写该数，同时去除同一行列宫的该草稿数（行列宫排除法）
    /// 3. 在某个宫内，某草稿数值占据了同一行列，可按行列方向排除其他宫内的该草稿数值（区块排除法）
    /// 4. 在某一行列宫内，存在二数对时，其他空行内可去除这些数对（二数对排除法）
    /// 5. 暂时先处理上述情况
    pub fn search_one_inference(&self) -> Option<Inference> {
        let inferences: Vec<FnInference> = vec![
            Field::inference_only_one_left,
            Field::inference_only_one_right,
            // Field::inference_only_one_right_ex,
            // Field::inference_cell_pair,
        ];
        for fn_inference in inferences {
            let opt = fn_inference(&self);
            if opt.is_none() {
                println!("fn_inference None");
                continue;
            }
            println!("fn_inference Some: {}", opt.as_ref().unwrap());
            return opt;
        }
        None
    }

    // 应用一个操作，为了实现“历史记录“功能，返回值是一个新的Field
    pub fn apply_one_inference(&self, inference: Inference) -> Field {
        let mut ret: Field = self.clone();
        for op in inference.conclusion {
            if op.situation == Situation::SetValue {
                ret.get_cell_mut_by_rc(op.cell.rc).value = op.value.unwrap();
                ret.get_cell_mut_by_rc(op.cell.rc).status = CellStatus::SOLVE;
            } else if op.situation == Situation::RemoveDrafts {
                ret.get_cell_mut_by_rc(op.cell.rc)
                    .drafts
                    .remove_draft(op.value.unwrap());
            }
        }
        ret
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut field = Field::initial_by_string(
            &"070009800008002006906100000600000150030801020092000008000003701800600300001900060"
                .to_string(),
        )
        .unwrap();
        field.print();
        // println!("{:?}", field.find_conflict());
        // println!("{:?}", field.find_empty_drafts());

        for _ in 0..40 {
            let inteference = field.search_one_inference();
            field = field.apply_one_inference(inteference.unwrap());
        }
        field.print();
    }
}
