use std::vec;

use crate::types::{Cell, CellStatus, CellValue, Field};

#[derive(Copy, Clone, Debug)]
pub struct CellAndValue<'a> {
    cell: &'a Cell,
    value: CellValue,
}

#[derive(Debug)]
pub struct Inference<'a> {
    inference_type: InferenceType,
    condition: Vec<CellAndValue<'a>>,
    conclusion_set_value: Option<Vec<CellAndValue<'a>>>,
    conclusion_remove_drafts: Option<Vec<CellAndValue<'a>>>,
}

type FnInference = fn(&Field) -> Option<Inference>;

#[derive(Debug)]
pub enum InferenceType {
    OnlyOneLeft,
    OnlyOneRightInRow,
    // OnlyOneRightInCol,
    // OnlyOneRightInGrid,
    // LockedCandidatesInRow,
    // LockedCandidatesInCol,
    // LockedCandidatesInGridByRow,
    // LockedCandidatesInGridByCol,
    // NakedPairInRow,
    // NakedPairInCol,
    // NakedPairInGrid,
    // NakedTripleInRow,
    // NakedTripleInCol,
    // NakedTripleInGrid,
    // NakedQuadrupleInRow,
    // NakedQuadrupleInCol,
    // NakedQuadrupleInGrid,
}

pub struct Inferences {
    inferences: Vec<(InferenceType, FnInference)>,
}
impl Inferences {
    pub fn search<'a>(field: &'a Field) -> Option<Inference> {
        let vecfn: Vec<FnInference> = vec![search_only_one_left, search_only_one_right_in_row];
        vecfn.iter().find_map(|fn_t| fn_t(field))
    }
}

// impl std::fmt::Display for Inference {
//     /// # 实现调试接口
//     /// R行
//     /// C列
//     /// G宫
//     /// N宫内序号
//     /// D草稿
//     /// V值
//     /// -去除
//     /// +设置值
//     /// &条件与、结论与
//     /// ^因为
//     /// =推导
//     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//         if self.condition.len() != 0 {
//             write!(f, "因为 ")?;
//             for (index, c) in self.condition.iter().enumerate() {
//                 write!(
//                     f,
//                     "R{}C{}V{}",
//                     c.cell.rc.r,
//                     c.cell.rc.c,
//                     (c.value.unwrap().to_index().unwrap() + 1)
//                 )?;
//                 if index < self.condition.len() - 1 {
//                     write!(f, " 和 ")?;
//                 }
//             }
//         }
//         if self.conclusion.len() != 0 {
//             write!(f, " ，推导出： ")?;
//             for (index, c) in self.conclusion.iter().enumerate() {
//                 if c.situation == Situation::SetValue {
//                     write!(
//                         f,
//                         "R{}C{}V{}",
//                         c.cell.rc.r,
//                         c.cell.rc.c,
//                         (c.value.unwrap().to_index().unwrap() + 1)
//                     )?;
//                 } else if c.situation == Situation::RemoveDrafts {
//                     write!(
//                         f,
//                         "R{}C{}-D{}",
//                         c.cell.rc.r,
//                         c.cell.rc.c,
//                         (c.value.unwrap().to_index().unwrap() + 1)
//                     )?;
//                 }
//                 if index < self.conclusion.len() - 1 {
//                     write!(f, " 和 ")?;
//                 }
//             }
//         }
//         write!(f, "")
//     }
// }

// 当某个格子设置某个值的时候，将同行列宫的该值的草稿值移除，输入值在vec_set_value.cells内，且value唯一
fn make_removing_drafts_when_set_value<'a>(
    field: &'a Field,
    vec_set_value: CellAndValue<'a>,
) -> Option<Vec<CellAndValue<'a>>> {
    let ret: Vec<CellAndValue> = field
        .collect_all_drafts_cells_in_r(vec_set_value.cell.rc.r)
        .iter()
        .filter_map(|&p| {
            if p.drafts.is_contain(vec_set_value.value) && (p.rc.c != vec_set_value.cell.rc.c) {
                Some(CellAndValue {
                    cell: p,
                    value: vec_set_value.value,
                })
            } else {
                None
            }
        })
        .chain(
            field
                .collect_all_drafts_cells_in_c(vec_set_value.cell.rc.c)
                .iter()
                .filter_map(|&p| {
                    if p.drafts.is_contain(vec_set_value.value)
                        && (p.rc.r != vec_set_value.cell.rc.r)
                    {
                        Some(CellAndValue {
                            cell: p,
                            value: vec_set_value.value,
                        })
                    } else {
                        None
                    }
                }),
        )
        .chain(
            field
                .collect_all_drafts_cells_in_g(vec_set_value.cell.gn.g)
                .iter()
                .filter_map(|&p| {
                    /* 需要去除同行列匹配的内容 */
                    if p.drafts.is_contain(vec_set_value.value)
                        && (p.rc.r != vec_set_value.cell.rc.r)
                        && (p.rc.c != vec_set_value.cell.rc.c)
                    {
                        Some(CellAndValue {
                            cell: p,
                            value: vec_set_value.value,
                        })
                    } else {
                        None
                    }
                }),
        )
        .collect();

    if ret.len() != 0 {
        Some(ret)
    } else {
        None
    }
}

// 唯余法，遍历所有草稿单元格，如果存在唯一草稿，则说明这个草稿填写该数字
fn search_only_one_left<'a>(field: &'a Field) -> Option<Inference> {
    field
        .collect_all_drafts_cells()
        .iter()
        .find_map(|&p| {
            (*p).drafts
                .try_get_the_only_one()
                .and_then(|cv| Some(CellAndValue { cell: p, value: cv }))
        })
        .and_then(move |ret| {
            Some({
                Inference {
                    inference_type: InferenceType::OnlyOneLeft,
                    condition: vec![ret],
                    conclusion_set_value: Some(vec![ret]),
                    conclusion_remove_drafts: make_removing_drafts_when_set_value(field, ret),
                }
            })
        })
}

// 按行排除法，每行中如果存在唯一草稿值，则填写该值，同时去除其余同一列宫的草稿值
fn search_only_one_right_in_row<'a>(field: &'a Field) -> Option<Inference> {
    // [r,c]->只要有一行有就行，返回值Option<CellAndValue> find_map(|| 条件只要有一个满足)
    // c([rv,d])->只要有一个格子满足，返回值Option<CellAndValue{cell:p,value:v}>，find_map(||)

    field
        .collect_all_drafts_cells_by_rc()
        .iter()
        .find_map(|vr| {
            vr.iter().find_map(|&p| {
                p.drafts
                    .to_vec()
                    .iter()
                    .find(|&v| {
                        /* 遍历当前行，在当前行中，除了当前格子外，其他格子不存在相同草稿值 */
                        vr.iter()
                            .copied()
                            .any(|tmp_p| (!(tmp_p.drafts.is_contain(*v))) && (tmp_p.rc.c != p.rc.c))
                    })
                    .and_then(|&ret| {
                        Some(CellAndValue {
                            cell: p,
                            value: ret,
                        })
                    })
            })
        })
        .and_then(|ret| {
            Some(Inference {
                inference_type: InferenceType::OnlyOneRightInRow,
                condition: vec![ret],
                conclusion_set_value: Some(vec![ret]),
                conclusion_remove_drafts: make_removing_drafts_when_set_value(field, ret),
            })
        })
}

// pub fn inference_only_one_right_in_row(&self) -> Option<Inference> {
//     let mut ret = Inference {
//         condition: vec![],
//         conclusion: vec![],
//     };
//     for r in 0..9 {
//         for c in 0..9 {
//             let p = self.get_cell_ref_by_rc(RCCoords { r, c });
//             if p.status == CellStatus::DRAFT {
//                 for (_, d) in p.drafts.to_vec().iter().enumerate() {
//                     let mut flag = true;
//                     for c_iter in 0..9 {
//                         if c_iter == c {
//                             continue;
//                         }
//                         let p_iter = self.get_cell_ref_by_rc(RCCoords { r: r, c: c_iter });
//                         if p_iter.status == CellStatus::DRAFT {
//                             if p_iter.drafts.is_contain(*d) {
//                                 flag = false;
//                                 break;
//                             }
//                         }
//                     }
//                     // 该草稿唯一
//                     if flag {
//                         ret.condition.push(Operator {
//                             situation: Situation::OnlyOneRightInRow,
//                             cell: p,
//                             value: Some(*d),
//                             drafts: None,
//                         });
//                         let sv_op = Operator {
//                             situation: Situation::SetValue,
//                             cell: p,
//                             value: Some(*d),
//                             drafts: None,
//                         };
//                         ret.conclusion.append(
//                             &mut Self::make_conclusion_with_remove_drafts_when_set_value(
//                                 self, sv_op,
//                             ),
//                         );
//                         return Some(ret);
//                     }
//                 }
//             }
//         }
//     }
//     None
// }

// // 排除法，对于每个格子内的草稿值，按照每行、每列、每宫方向进行判断，如果唯一，则填写该值，同时去除其余同一行列宫的草稿值
// pub fn inference_only_one_right_in_col(&self) -> Option<Inference> {
//     let mut ret = Inference {
//         condition: vec![],
//         conclusion: vec![],
//     };
//     for r in 0..9 {
//         for c in 0..9 {
//             let p = self.get_cell_ref_by_rc(RCCoords { r: r, c: c });
//             if p.status == CellStatus::DRAFT {
//                 for (_, d) in p.drafts.to_vec().iter().enumerate() {
//                     let mut flag = true;
//                     for r_iter in 0..9 {
//                         if r_iter == r {
//                             continue;
//                         }
//                         let p_iter = self.get_cell_ref_by_rc(RCCoords { r: r_iter, c: c });
//                         if p_iter.status == CellStatus::DRAFT {
//                             if p_iter.drafts.is_contain(*d) {
//                                 flag = false;
//                                 break;
//                             }
//                         }
//                     }
//                     // 该草稿唯一
//                     if flag {
//                         ret.condition.push(Operator {
//                             situation: Situation::OnlyOneRightInCol,
//                             cell: p,
//                             value: Some(*d),
//                             drafts: None,
//                         });
//                         let sv_op = Operator {
//                             situation: Situation::SetValue,
//                             cell: p,
//                             value: Some(*d),
//                             drafts: None,
//                         };
//                         ret.conclusion.append(
//                             &mut Self::make_conclusion_with_remove_drafts_when_set_value(
//                                 self, sv_op,
//                             ),
//                         );
//                         return Some(ret);
//                     }
//                 }
//             }
//         }
//     }
//     None
// }

// // 排除法，对于每个格子内的草稿值，按照每行、每列、每宫方向进行判断，如果唯一，则填写该值，同时去除其余同一行列宫的草稿值
// pub fn inference_only_one_right_in_grid(&self) -> Option<Inference> {
//     let mut ret = Inference {
//         condition: vec![],
//         conclusion: vec![],
//     };
//     for r in 0..9 {
//         for c in 0..9 {
//             let p = self.get_cell_ref_by_rc(RCCoords { r: r, c: c });
//             if p.status == CellStatus::DRAFT {
//                 for (_, d) in p.drafts.to_vec().iter().enumerate() {
//                     let mut flag = true;
//                     for n_iter in 0..9 {
//                         if n_iter == p.gn.n {
//                             continue;
//                         }
//                         let p_iter = self.get_cell_ref_by_gn(GNCoords {
//                             g: p.gn.g,
//                             n: n_iter,
//                         });
//                         if p_iter.status == CellStatus::DRAFT {
//                             if p_iter.drafts.is_contain(*d) {
//                                 flag = false;
//                                 break;
//                             }
//                         }
//                     }
//                     // 该草稿唯一
//                     if flag {
//                         ret.condition.push(Operator {
//                             situation: Situation::OnlyOneRightInCol,
//                             cell: p,
//                             value: Some(*d),
//                             drafts: None,
//                         });
//                         let sv_op = Operator {
//                             situation: Situation::SetValue,
//                             cell: p,
//                             value: Some(*d),
//                             drafts: None,
//                         };
//                         ret.conclusion.append(
//                             &mut Self::make_conclusion_with_remove_drafts_when_set_value(
//                                 self, sv_op,
//                             ),
//                         );
//                         return Some(ret);
//                     }
//                 }
//             }
//         }
//     }
//     None
// }

// // 高级排除法1
// // 当一宫内的某种草稿值当且仅当在同一行/列时（其他行列不能有），可以排除行/列内其余格子的该草稿值
// pub fn inference_only_one_right_ex1(&self) -> Option<Inference> {
//     let mut ret = Inference {
//         condition: vec![],
//         conclusion: vec![],
//     };
//     for g_iter in 0..9 {
//         'v_iter: for v_iter in CellValue::vec_for_iter() {
//             let mut same_p_set: Vec<&Cell> = vec![];
//             'n_iter: for n_iter in 0..9 {
//                 let p = self.get_cell_ref_by_gn(GNCoords {
//                     g: g_iter,
//                     n: n_iter,
//                 });
//                 if p.status == CellStatus::FIXED || p.status == CellStatus::SOLVE {
//                     if p.value == v_iter {
//                         continue 'v_iter;
//                     }
//                     continue 'n_iter;
//                 } else if p.status == CellStatus::DRAFT {
//                     if p.drafts.is_contain(v_iter) {
//                         same_p_set.push(p);
//                     }
//                 }
//             }
//             if same_p_set.len() < 2 {
//                 continue 'v_iter;
//             } else {
//                 // 判断是否在同一行
//                 {
//                     let tmp_r = same_p_set[0].rc.r;
//                     let mut flag = true;
//                     for i in 1..same_p_set.len() {
//                         if same_p_set[i].rc.r != tmp_r {
//                             flag = false;
//                             break;
//                         }
//                     }
//                     if flag {
//                         // same_p_set 数组的长度必然大于2，到这里说明符合判断条件
//                         // 寻找相同行的值是否存在该草稿数，需要排除相同宫的值
//                         let mut tmp_ret: Vec<&Cell> = vec![];
//                         for c_iter in 0..9 {
//                             let p_iter = self.get_cell_ref_by_rc(RCCoords {
//                                 r: tmp_r,
//                                 c: c_iter,
//                             });
//                             if p_iter.gn.g != g_iter
//                                 && p_iter.status == CellStatus::DRAFT
//                                 && p_iter.drafts.is_contain(v_iter)
//                             {
//                                 tmp_ret.push(p_iter);
//                             }
//                         }
//                         if tmp_ret.len() != 0 {
//                             for item in same_p_set.iter() {
//                                 ret.condition.push(Operator {
//                                     situation: Situation::LockedCandidatesInGridByRow,
//                                     cell: item,
//                                     value: Some(v_iter),
//                                     drafts: None,
//                                 })
//                             }
//                             for item in tmp_ret.iter() {
//                                 ret.conclusion.push(Operator {
//                                     situation: Situation::RemoveDrafts,
//                                     cell: item,
//                                     value: Some(v_iter),
//                                     drafts: None,
//                                 })
//                             }
//                             return Some(ret);
//                         }
//                     }
//                 }
//                 // 判断是否在同一列
//                 {
//                     let tmp_c = same_p_set[0].rc.c;
//                     let mut flag = true;
//                     for i in 1..same_p_set.len() {
//                         if same_p_set[i].rc.c != tmp_c {
//                             flag = false;
//                             break;
//                         }
//                     }
//                     if flag {
//                         // same_p_set 数组的长度必然大于2，到这里说明符合判断条件
//                         // 寻找相同行的值是否存在该草稿数，需要排除相同宫的值
//                         let mut tmp_ret: Vec<&Cell> = vec![];
//                         for r_iter in 0..9 {
//                             let p_iter = self.get_cell_ref_by_rc(RCCoords {
//                                 r: r_iter,
//                                 c: tmp_c,
//                             });
//                             if p_iter.gn.g != g_iter
//                                 && p_iter.status == CellStatus::DRAFT
//                                 && p_iter.drafts.is_contain(v_iter)
//                             {
//                                 tmp_ret.push(p_iter);
//                             }
//                         }
//                         if tmp_ret.len() != 0 {
//                             for item in same_p_set.iter() {
//                                 ret.condition.push(Operator {
//                                     situation: Situation::LockedCandidatesInGridByCol,
//                                     cell: item,
//                                     value: Some(item.value),
//                                     drafts: None,
//                                 })
//                             }
//                             for item in tmp_ret.iter() {
//                                 ret.conclusion.push(Operator {
//                                     situation: Situation::RemoveDrafts,
//                                     cell: item,
//                                     value: Some(item.value),
//                                     drafts: None,
//                                 })
//                             }
//                             return Some(ret);
//                         }
//                     }
//                 }
//             }
//         }
//     }

//     None
// }

// // 高级排除法2
// // 当一行列的草稿数正好在1宫时，排除该宫其他草稿数
// pub fn inference_only_one_right_ex2(&self) -> Option<Inference> {
//     let mut ret = Inference {
//         condition: vec![],
//         conclusion: vec![],
//     };
//     // 按行遍历
//     for r_iter in 0..9 {
//         'v_iter: for v_iter in CellValue::vec_for_iter() {
//             let mut same_p_set: Vec<&Cell> = vec![];
//             'c_iter: for c_iter in 0..9 {
//                 let p = self.get_cell_ref_by_rc(RCCoords {
//                     r: r_iter,
//                     c: c_iter,
//                 });
//                 if p.status == CellStatus::FIXED || p.status == CellStatus::SOLVE {
//                     if p.value == v_iter {
//                         continue 'v_iter;
//                     }
//                     continue 'c_iter;
//                 } else if p.status == CellStatus::DRAFT {
//                     if p.drafts.is_contain(v_iter) {
//                         same_p_set.push(p);
//                     }
//                 }
//             }
//             if same_p_set.len() < 2 {
//                 continue 'v_iter;
//             } else {
//                 // 判断是否在同一宫
//                 {
//                     let tmp_g = same_p_set[0].gn.g;
//                     let mut flag = true;
//                     for i in 1..same_p_set.len() {
//                         if same_p_set[i].gn.g != tmp_g {
//                             flag = false;
//                             break;
//                         }
//                     }
//                     if flag {
//                         // same_p_set 数组的长度必然大于2，到这里说明符合判断条件
//                         // 寻找相同宫的值是否存在该草稿数，需要排除相同宫的值
//                         let mut tmp_ret: Vec<&Cell> = vec![];
//                         for n_iter in 0..9 {
//                             let p_iter = self.get_cell_ref_by_gn(GNCoords {
//                                 g: tmp_g,
//                                 n: n_iter,
//                             });
//                             if p_iter.rc.r != r_iter
//                                 && p_iter.status == CellStatus::DRAFT
//                                 && p_iter.drafts.is_contain(v_iter)
//                             {
//                                 tmp_ret.push(p_iter);
//                             }
//                         }
//                         if tmp_ret.len() != 0 {
//                             for item in same_p_set.iter() {
//                                 ret.condition.push(Operator {
//                                     situation: Situation::LockedCandidatesInRow,
//                                     cell: item,
//                                     value: Some(v_iter),
//                                     drafts: None,
//                                 })
//                             }
//                             for item in tmp_ret.iter() {
//                                 ret.conclusion.push(Operator {
//                                     situation: Situation::RemoveDrafts,
//                                     cell: item,
//                                     value: Some(v_iter),
//                                     drafts: None,
//                                 })
//                             }
//                             return Some(ret);
//                         }
//                     }
//                 }
//             }
//         }
//     }

//     // 按列遍历
//     for c_iter in 0..9 {
//         'v_iter: for v_iter in CellValue::vec_for_iter() {
//             let mut same_p_set: Vec<&Cell> = vec![];
//             'r_iter: for r_iter in 0..9 {
//                 let p = self.get_cell_ref_by_rc(RCCoords {
//                     r: r_iter,
//                     c: c_iter,
//                 });
//                 if p.status == CellStatus::FIXED || p.status == CellStatus::SOLVE {
//                     if p.value == v_iter {
//                         continue 'v_iter;
//                     }
//                     continue 'r_iter;
//                 } else if p.status == CellStatus::DRAFT {
//                     if p.drafts.is_contain(v_iter) {
//                         same_p_set.push(p);
//                     }
//                 }
//             }
//             if same_p_set.len() < 2 {
//                 continue 'v_iter;
//             } else {
//                 // 判断是否在同一宫
//                 {
//                     let tmp_g = same_p_set[0].gn.g;
//                     let mut flag = true;
//                     for i in 1..same_p_set.len() {
//                         if same_p_set[i].gn.g != tmp_g {
//                             flag = false;
//                             break;
//                         }
//                     }
//                     if flag {
//                         // same_p_set 数组的长度必然大于2，到这里说明符合判断条件
//                         // 寻找相同宫的值是否存在该草稿数，需要排除相同宫的值
//                         let mut tmp_ret: Vec<&Cell> = vec![];
//                         for n_iter in 0..9 {
//                             let p_iter = self.get_cell_ref_by_gn(GNCoords {
//                                 g: tmp_g,
//                                 n: n_iter,
//                             });
//                             if p_iter.rc.c != c_iter
//                                 && p_iter.status == CellStatus::DRAFT
//                                 && p_iter.drafts.is_contain(v_iter)
//                             {
//                                 tmp_ret.push(p_iter);
//                             }
//                         }
//                         if tmp_ret.len() != 0 {
//                             for item in same_p_set.iter() {
//                                 ret.condition.push(Operator {
//                                     situation: Situation::LockedCandidatesInCol,
//                                     cell: item,
//                                     value: Some(item.value),
//                                     drafts: None,
//                                 })
//                             }
//                             for item in tmp_ret.iter() {
//                                 ret.conclusion.push(Operator {
//                                     situation: Situation::RemoveDrafts,
//                                     cell: item,
//                                     value: Some(item.value),
//                                     drafts: None,
//                                 })
//                             }
//                             return Some(ret);
//                         }
//                     }
//                 }
//             }
//         }
//     }

//     None
// }

// // 显性2数对排除法
// // 在同一行/列/宫内，存在2个格子的数字数量为2且相同，则该格子为2数对，同行内其余该草稿数可以被移除
// pub fn inference_cell_naked_pair_in_row(&self) -> Option<Inference> {
//     for r_iter in 0..9 {
//         let mut avail_v_set: Vec<&CellValue> = vec![];
//     }
//     None
// }

// /// # 数独推理过程
// /// 1. 对每个格子判断唯一性，当只有1个候选数时，该格子必定为此数，填写该数，同时去除同一行列宫的该草稿数（唯余法）
// /// 2. 对行列宫判断唯一性，当一个候选数在同一行列宫只有唯一选项时，填写该数，同时去除同一行列宫的该草稿数（行列宫排除法）
// /// 3. 在某个宫内，某草稿数值占据了同一行列，可按行列方向排除其他宫内的该草稿数值（区块排除法）
// /// 4. 在某一行列宫内，存在二数对时，其他空行内可去除这些数对（二数对排除法）
// /// 5. 暂时先处理上述情况
// pub fn search_one_inference(&self) -> Option<Inference> {
//     let inferences: Vec<FnInference> = vec![
//         Field::inference_only_one_left,
//         Field::inference_only_one_right_in_row,
//         Field::inference_only_one_right_in_col,
//         Field::inference_only_one_right_in_grid,
//         Field::inference_only_one_right_ex1,
//         Field::inference_only_one_right_ex2,
//         Field::inference_cell_naked_pair_in_row,
//         // Field::inference_cell_naked_triple,
//         // Field::inference_cell_naked_quadruple,
//         // Field::inference_cell_hidden_pair,
//         // Field::inference_cell_hidden_triple,
//         // Field::inference_cell_hidden_quadruple,
//     ];
//     for fn_inference in inferences {
//         let opt = fn_inference(&self);
//         if opt.is_none() {
//             // println!("fn_inference None");
//             continue;
//         }
//         println!("{}", opt.as_ref().unwrap());
//         return opt;
//     }
//     None
// }

// // 应用一个操作，为了实现“历史记录“功能，返回值是一个新的Field
// pub fn apply_one_inference(&self, inference: Inference) -> Field {
//     let mut ret: Field = self.clone();
//     for op in inference.conclusion {
//         if op.situation == Situation::SetValue {
//             ret.get_cell_mut_by_rc(op.cell.rc).value = op.value.unwrap();
//             ret.get_cell_mut_by_rc(op.cell.rc).status = CellStatus::SOLVE;
//         } else if op.situation == Situation::RemoveDrafts {
//             ret.get_cell_mut_by_rc(op.cell.rc)
//                 .drafts
//                 .remove_draft(op.value.unwrap());
//         }
//     }
//     ret
// }
