use crate::types::{
    Cell, CellStatus, CellValue, Coords, Field, GNCoords, RCCoords, TheCellAndTheValue,
};

#[derive(Clone)]
pub struct InferenceResult<'a> {
    condition: Vec<TheCellAndTheValue<'a>>,
    conclusion_set_value: Option<Vec<TheCellAndTheValue<'a>>>,
    conclusion_remove_drafts: Option<Vec<TheCellAndTheValue<'a>>>,
}

trait Inference {
    fn analyze(field: &Field) -> Option<InferenceResult>
    where
        Self: Sized;
    fn write_result(&self, inference_result: &InferenceResult) -> String;
}

// type FnInference = fn(&Field) -> Option<Inference>;

// #[derive(Debug)]
// pub enum InferenceType {
//     OnlyOneLeft,
//     OnlyOneRightInRow,
//     OnlyOneRightInCol,
//     OnlyOneRightInGrid,
//     LockedCandidatesInRowByGrid,
//     LockedCandidatesInColByGrid,
//     LockedCandidatesInGridByRow,
//     LockedCandidatesInGridByCol,
//     NakedPairInRow,
//     NakedPairInCol,
//     NakedPairInGrid,
//     NakedTripleInRow,
//     NakedTripleInCol,
//     NakedTripleInGrid,
//     NakedQuadrupleInRow,
//     NakedQuadrupleInCol,
//     NakedQuadrupleInGrid,
// }

pub struct InferenceSet {
    inferences: Vec<Box<dyn Inference>>,
}

impl InferenceSet {
    pub fn new() -> Self {
        InferenceSet {
            inferences: vec![Box::new(OnlyOneLeftInference)],
        }
    }

    pub fn analyze(&self, field: &Field) -> Option<InferenceResult> {
        self.inferences.iter().find_map(|&inf| { Box<dyn Inference>::analyze(field)})
    }

    pub fn apply(field: &Field, result: InferenceResult) -> Field {
        let mut ret = field.clone();
        if result.conclusion_set_value.is_some() {
            result.conclusion_set_value.unwrap().iter().for_each(|cv| {
                let p = ret.get_cell_mut_by_coords(Coords::RC((cv.the_cell).rc));
                p.value = cv.the_value;
                p.status = CellStatus::SOLVE;
            })
        };
        if result.conclusion_remove_drafts.is_some() {
            result
                .conclusion_remove_drafts
                .unwrap()
                .iter()
                .for_each(|cv| {
                    let p = ret.get_cell_mut_by_coords(Coords::RC((cv.the_cell).rc));
                    p.drafts.remove_draft(cv.the_value);
                })
        }
        ret
    }
}

// impl std::fmt::Debug for Inference {
//     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//         write!(f, "因为 ")?;

//         match self.inference_type {
//             InferenceType::OnlyOneLeft => {}
//             InferenceType::OnlyOneRightInRow => {
//                 write!(
//                     f,
//                     "{:?} 的 {:?} 在 R{:?} 内唯一，推导出：这里只能填写 {:?} ",
//                     self.condition[0].the_coords,
//                     self.condition[0].the_value,
//                     self.condition[0].the_coords.to_rc_coords().r + 1,
//                     self.condition[0].the_value
//                 )?;

//                 if self.conclusion_remove_drafts.is_some() {
//                     write!(f, "，且移除 ")?;
//                     self.conclusion_remove_drafts
//                         .as_ref()
//                         .unwrap()
//                         .iter()
//                         .for_each(|cv| {
//                             write!(f, "{:?} ", cv.the_coords).unwrap();
//                         });
//                     write!(f, "的可能 {:?}", self.condition[0].the_value)?;
//                 }
//                 write!(f, "。")?;
//             }
//             InferenceType::OnlyOneRightInCol => {
//                 write!(
//                     f,
//                     "{:?} 的 {:?} 在 C{:?} 内唯一，推导出：这里只能填写 {:?} ",
//                     self.condition[0].the_coords,
//                     self.condition[0].the_value,
//                     self.condition[0].the_coords.to_rc_coords().c + 1,
//                     self.condition[0].the_value
//                 )?;

//                 if self.conclusion_remove_drafts.is_some() {
//                     write!(f, "，且移除 ")?;
//                     self.conclusion_remove_drafts
//                         .as_ref()
//                         .unwrap()
//                         .iter()
//                         .for_each(|cv| {
//                             write!(f, "{:?} ", cv.the_coords).unwrap();
//                         });
//                     write!(f, "的可能 {:?}", self.condition[0].the_value)?;
//                 }
//                 write!(f, "。")?;
//             }
//             InferenceType::OnlyOneRightInGrid => {
//                 write!(
//                     f,
//                     "{:?} 的 {:?} 在 G{:?} 内唯一，推导出：这里只能填写 {:?} ",
//                     self.condition[0].the_coords.to_gn_coords(),
//                     self.condition[0].the_value,
//                     self.condition[0].the_coords.to_gn_coords().g + 1,
//                     self.condition[0].the_value
//                 )?;

//                 if self.conclusion_remove_drafts.is_some() {
//                     write!(f, "，且移除 ")?;
//                     self.conclusion_remove_drafts
//                         .as_ref()
//                         .unwrap()
//                         .iter()
//                         .for_each(|cv| {
//                             write!(f, "{:?} ", cv.the_coords).unwrap();
//                         });
//                     write!(f, "的可能 {:?}", self.condition[0].the_value)?;
//                 }
//                 write!(f, "。")?;
//             }
//             InferenceType::LockedCandidatesInRowByGrid => {
//                 self.condition.iter().for_each(|cv| {
//                     write!(f, "{:?} ", cv.the_cell.rc).unwrap();
//                 });
//                 write!(
//                     f,
//                     "的 {:?} 在 G{:?} 内，只在 R{:?} 中存在，推导出： ",
//                     self.condition[0].the_value,
//                     self.condition[0].the_cell.gn.g + 1,
//                     self.condition[0].the_cell.rc.r + 1
//                 )?;

//                 self.conclusion_remove_drafts
//                     .as_ref()
//                     .unwrap()
//                     .iter()
//                     .for_each(|p| {
//                         write!(f, "{:?} ", p.the_cell.rc).unwrap();
//                     });
//                 write!(f, "不能填写 {:?} ，需要移除。", self.condition[0].the_value)?;
//             }
//             InferenceType::LockedCandidatesInColByGrid => {
//                 self.condition.iter().for_each(|cv| {
//                     write!(f, "{:?} ", cv.the_cell.gn,).unwrap();
//                 });
//                 write!(
//                     f,
//                     "的 {:?} 在宫 G{:?} 内，只在 C{:?} 中存在，推导出： ",
//                     self.condition[0].the_value,
//                     self.condition[0].the_cell.gn.g + 1,
//                     self.condition[0].the_cell.rc.c + 1
//                 )?;

//                 self.conclusion_remove_drafts
//                     .as_ref()
//                     .unwrap()
//                     .iter()
//                     .for_each(|p| {
//                         write!(f, "{:?} ", p.the_cell.rc).unwrap();
//                     });
//                 write!(f, "不能填写 {:?} ，需要移除。", self.condition[0].the_value)?;
//             }
//             InferenceType::LockedCandidatesInGridByRow => {
//                 self.condition.iter().for_each(|cv| {
//                     write!(f, "{:?} ", cv.the_cell.rc).unwrap();
//                 });
//                 write!(
//                     f,
//                     "的 {:?} 在 R{:?} 内，只在宫 G{:?} 中存在，推导出： ",
//                     self.condition[0].the_value,
//                     self.condition[0].the_cell.rc.r + 1,
//                     self.condition[0].the_cell.gn.g + 1
//                 )?;

//                 self.conclusion_remove_drafts
//                     .as_ref()
//                     .unwrap()
//                     .iter()
//                     .for_each(|p| {
//                         write!(f, "{:?} ", p.the_cell.gn).unwrap();
//                     });
//                 write!(f, "不能填写 {:?} ，需要移除。", self.condition[0].the_value)?;
//             }
//             InferenceType::LockedCandidatesInGridByCol => {
//                 self.condition.iter().for_each(|cv| {
//                     write!(f, "{:?} ", cv.the_cell.rc).unwrap();
//                 });
//                 write!(
//                     f,
//                     "的 {:?} 在 C{:?} 内，只在宫 G{:?} 中存在，推导出： ",
//                     self.condition[0].the_value,
//                     self.condition[0].the_cell.rc.c + 1,
//                     self.condition[0].the_cell.gn.g + 1
//                 )?;

//                 self.conclusion_remove_drafts
//                     .as_ref()
//                     .unwrap()
//                     .iter()
//                     .for_each(|p| {
//                         write!(f, "{:?} ", p.the_cell.gn).unwrap();
//                     });
//                 write!(f, "不能填写 {:?} ，需要移除。", self.condition[0].the_value)?;
//             }
//             InferenceType::NakedPairInRow => todo!(),
//             InferenceType::NakedPairInCol => todo!(),
//             InferenceType::NakedPairInGrid => todo!(),
//             InferenceType::NakedTripleInRow => todo!(),
//             InferenceType::NakedTripleInCol => todo!(),
//             InferenceType::NakedTripleInGrid => todo!(),
//             InferenceType::NakedQuadrupleInRow => todo!(),
//             InferenceType::NakedQuadrupleInCol => todo!(),
//             InferenceType::NakedQuadrupleInGrid => todo!(),
//         }

//         write!(f, "")
//     }
// }

/// 唯余法，遍历所有草稿单元格，如果存在唯一草稿，则说明这个草稿填写该数字
struct OnlyOneLeftInference;
impl Inference for OnlyOneLeftInference {
    fn analyze(field: &Field) -> Option<InferenceResult> {
        field
            .collect_all_drafts_cells()
            .iter()
            .find_map(|&p| {
                (*p).drafts.try_get_the_only_one().and_then(|cv| {
                    Some(TheCellAndTheValue {
                        the_cell: p,
                        the_value: cv,
                    })
                })
            })
            .and_then(move |ret| {
                Some({
                    InferenceResult {
                        condition: vec![ret.clone()],
                        conclusion_set_value: Some(vec![ret.clone()]),
                        conclusion_remove_drafts: field
                            .collect_all_drafts_coords_by_the_coords_and_the_value(
                                ret.the_cell,
                                ret.the_value,
                            ),
                    }
                })
            })
    }

    fn write_result(&self, inference_result: &InferenceResult) -> String {
        let mut r = format!(
            "{:?} 的可能 {:?} 在格内唯一，推导出：这里只能填写 {:?} ",
            inference_result.condition[0].the_cell.rc,
            inference_result.condition[0].the_value,
            inference_result.condition[0].the_value
        );

        if inference_result.conclusion_remove_drafts.is_some() {
            r.push_str(&format!("，且移除 "));
            inference_result
                .conclusion_remove_drafts
                .as_ref()
                .unwrap()
                .iter()
                .for_each(|cv| {
                    r.push_str(&format!("{:?} ", cv.the_cell.rc));
                });
            r.push_str(&format!(
                "的可能 {:?} ",
                inference_result.condition[0].the_value
            ));
        }

        r
    }
}

/// 按行排除法，每行中如果存在唯一草稿值，则填写该值，同时去除其余同一列宫的草稿值
struct OnlyOneRightInRowInference;
impl Inference for OnlyOneRightInRowInference {
    fn analyze(field: &Field) -> Option<InferenceResult> {
        field.iter_all_drafts_cells_by_rc().find_map(|vr| {
            vr.iter().find_map(|&p| {
                p.drafts
                    .to_vec()
                    .iter()
                    .find(|&v| {
                        !(vr.iter().fold(false, |acc, p_iter| {
                            acc || ((p_iter.rc.c != p.rc.c) && (p_iter.drafts.is_contain(*v)))
                        }))
                    })
                    .and_then(|&ret| {
                        let cv = TheCellAndTheValue {
                            the_cell: p,
                            the_value: ret,
                        };
                        Some(InferenceResult {
                            condition: vec![cv.clone()],
                            conclusion_set_value: Some(vec![cv]),
                            conclusion_remove_drafts: field
                                .collect_all_drafts_coords_by_the_coords_and_the_value(p, ret),
                        })
                    })
            })
        })
    }

    fn write_result(&self, inference_result: &InferenceResult) -> String {
        let mut r = format!(
            "{:?} 的可能 {:?} 在行 {:?} 内唯一，推导出：这里只能填写 {:?} ",
            inference_result.condition[0].the_cell.rc,
            inference_result.condition[0].the_value,
            inference_result.condition[0].the_cell.rc.r + 1,
            inference_result.condition[0].the_value
        );

        if inference_result.conclusion_remove_drafts.is_some() {
            r.push_str(&format!("，且移除 "));
            inference_result
                .conclusion_remove_drafts
                .as_ref()
                .unwrap()
                .iter()
                .for_each(|cv| {
                    r.push_str(&format!("{:?} ", cv.the_cell.rc));
                });
            r.push_str(&format!(
                "的可能 {:?} ",
                inference_result.condition[0].the_value
            ));
        }

        r
    }
}

// // 按列排除法，每行中如果存在唯一草稿值，则填写该值，同时去除其余同一列宫的草稿值
// fn search_only_one_right_in_col(field: &Field) -> Option<Inference> {
//     field
//         .collect_all_drafts_cells_by_cr()
//         .iter()
//         .find_map(|vc| {
//             vc.iter().find_map(|&p| {
//                 p.drafts
//                     .to_vec()
//                     .iter()
//                     .find(|&v| {
//                         !(vc.iter().fold(false, |acc, p_iter| {
//                             acc || ((p_iter.rc.r != p.rc.r) && (p_iter.drafts.is_contain(*v)))
//                         }))
//                     })
//                     .and_then(|&ret| {
//                         let cv = TheCoordsAndTheValue {
//                             the_cell: p,
//                             the_value: ret,
//                         };
//                         Some(Inference {
//                             inference_type: InferenceType::OnlyOneRightInCol,
//                             condition: vec![cv],
//                             conclusion_set_value: Some(vec![cv]),
//                             conclusion_remove_drafts: make_removing_drafts_when_set_value(
//                                 field, cv,
//                             ),
//                         })
//                     })
//             })
//         })
// }

// // 按宫排除法，每行中如果存在唯一草稿值，则填写该值，同时去除其余同一列宫的草稿值
// fn search_only_one_right_in_grid(field: &Field) -> Option<Inference> {
//     field
//         .collect_all_drafts_cells_by_gn()
//         .iter()
//         .find_map(|vg| {
//             vg.iter().find_map(|&p| {
//                 p.drafts
//                     .to_vec()
//                     .iter()
//                     .find(|&v| {
//                         !(vg.iter().fold(false, |acc, p_iter| {
//                             acc || ((p_iter.gn.n != p.gn.n) && (p_iter.drafts.is_contain(*v)))
//                         }))
//                     })
//                     .and_then(|&ret| {
//                         let cv = TheCoordsAndTheValue {
//                             the_cell: p,
//                             the_value: ret,
//                         };
//                         Some(Inference {
//                             inference_type: InferenceType::OnlyOneRightInGrid,
//                             condition: vec![cv],
//                             conclusion_set_value: Some(vec![cv]),
//                             conclusion_remove_drafts: make_removing_drafts_when_set_value(
//                                 field, cv,
//                             ),
//                         })
//                     })
//             })
//         })
// }

// // 当一宫内的某种草稿值当且仅当在同一行/列时，可以排除行/列内其余格子的该草稿值
// pub fn search_locked_candidates_in_row_col_by_grid(field: &Field) -> Option<Inference> {
//     field
//         .collect_all_drafts_cells_by_gn()
//         .iter()
//         .find_map(|vg| {
//             CellValue::vec_for_iter()
//                 .iter()
//                 .filter_map(|&v| {
//                     let tmp: Vec<&Cell> = vg
//                         .iter()
//                         .filter_map(|&p| p.drafts.is_contain(v).then_some(p))
//                         .collect::<Vec<&Cell>>();
//                     (tmp.len() != 0).then_some((v, tmp))
//                 })
//                 .find_map(|(v, vp)| {
//                     {
//                         let vr = field.collect_all_drafts_cells_in_r(vp[0].rc.r);
//                         // 条件1：该宫内其他行没有这个值
//                         let ret1 = !vp.iter().any(|&p| (p.rc.r != vp[0].rc.r));
//                         // 条件2：宫外该行内有这个值
//                         let ret2 = vr
//                             .iter()
//                             .filter_map(|&vr_p_iter| {
//                                 ((vr_p_iter.gn.g != vp[0].gn.g) && vr_p_iter.drafts.is_contain(v))
//                                     .then_some(vr_p_iter)
//                             })
//                             .collect::<Vec<&Cell>>();
//                         if ret1 && ret2.len() != 0 {
//                             Some(Inference {
//                                 inference_type: InferenceType::LockedCandidatesInRowByGrid,
//                                 condition: vp
//                                     .iter()
//                                     .map(|&p| TheCoordsAndTheValue {
//                                         the_cell: p,
//                                         the_value: v,
//                                     })
//                                     .collect::<Vec<TheCoordsAndTheValue>>(),
//                                 conclusion_set_value: None,
//                                 conclusion_remove_drafts: Some(
//                                     ret2.iter()
//                                         .map(|&p| TheCoordsAndTheValue {
//                                             the_cell: p,
//                                             the_value: v,
//                                         })
//                                         .collect::<Vec<TheCoordsAndTheValue>>(),
//                                 ),
//                             })
//                         } else {
//                             None
//                         }
//                     }
//                     .or({
//                         {
//                             let vc = field.collect_all_drafts_cells_in_c(vp[0].rc.c);
//                             // 条件1：该宫内其他列没有这个值
//                             let ret1 = !vp.iter().any(|&p| (p.rc.c != vp[0].rc.c));
//                             // 条件2：宫外该列内有这个值
//                             let ret2 = vc
//                                 .iter()
//                                 .filter_map(|&vc_p_iter| {
//                                     ((vc_p_iter.gn.g != vp[0].gn.g)
//                                         && vc_p_iter.drafts.is_contain(v))
//                                     .then_some(vc_p_iter)
//                                 })
//                                 .collect::<Vec<&Cell>>();
//                             (ret1 && ret2.len() != 0).then_some(Inference {
//                                 inference_type: InferenceType::LockedCandidatesInColByGrid,
//                                 condition: vp
//                                     .iter()
//                                     .map(|&p| TheCoordsAndTheValue {
//                                         the_cell: p,
//                                         the_value: v,
//                                     })
//                                     .collect::<Vec<TheCoordsAndTheValue>>(),
//                                 conclusion_set_value: None,
//                                 conclusion_remove_drafts: Some(
//                                     ret2.iter()
//                                         .map(|&p| TheCoordsAndTheValue {
//                                             the_cell: p,
//                                             the_value: v,
//                                         })
//                                         .collect::<Vec<TheCoordsAndTheValue>>(),
//                                 ),
//                             })
//                         }
//                     })
//                 })
//         })
// }

// // 当一行的草稿数正好在一宫时，排除该宫的其他草稿数
// pub fn search_locked_candidates_in_grid_by_row(field: &Field) -> Option<Inference> {
//     field
//         .collect_all_drafts_cells_by_rc()
//         .iter()
//         .find_map(|vr| {
//             CellValue::vec_for_iter()
//                 .iter()
//                 .filter_map(|&v| {
//                     let tmp: Vec<&Cell> = vr
//                         .iter()
//                         .filter_map(|&p| p.drafts.is_contain(v).then_some(p))
//                         .collect::<Vec<&Cell>>();
//                     (tmp.len() != 0).then_some((v, tmp))
//                 })
//                 .find_map(|(v, vp)| {
//                     {
//                         let vg = field.collect_all_drafts_cells_in_g(vp[0].gn.g);
//                         // 条件1：该行内的值都在同一个宫内
//                         let ret1 = !vp.iter().any(|&p| (p.gn.g != vp[0].gn.g));
//                         // 条件2：第一个值所在的宫，在其他行内有值
//                         let ret2 = vg
//                             .iter()
//                             .filter_map(|&vr_p_iter| {
//                                 ((vr_p_iter.rc.r != vp[0].rc.r) && vr_p_iter.drafts.is_contain(v))
//                                     .then_some(vr_p_iter)
//                             })
//                             .collect::<Vec<&Cell>>();
//                         if ret1 && ret2.len() != 0 {
//                             Some(Inference {
//                                 inference_type: InferenceType::LockedCandidatesInGridByRow,
//                                 condition: vp
//                                     .iter()
//                                     .map(|&p| TheCoordsAndTheValue {
//                                         the_cell: p,
//                                         the_value: v,
//                                     })
//                                     .collect::<Vec<TheCoordsAndTheValue>>(),
//                                 conclusion_set_value: None,
//                                 conclusion_remove_drafts: Some(
//                                     ret2.iter()
//                                         .map(|&p| TheCoordsAndTheValue {
//                                             the_cell: p,
//                                             the_value: v,
//                                         })
//                                         .collect::<Vec<TheCoordsAndTheValue>>(),
//                                 ),
//                             })
//                         } else {
//                             None
//                         }
//                     }
//                 })
//         })
// }

// // 当一列的草稿数正好在一宫时，排除该宫的其他草稿数
// pub fn search_locked_candidates_in_grid_by_col(field: &Field) -> Option<Inference> {
//     field
//         .collect_all_drafts_cells_by_cr()
//         .iter()
//         .find_map(|vc| {
//             CellValue::vec_for_iter()
//                 .iter()
//                 .filter_map(|&v| {
//                     let tmp: Vec<&Cell> = vc
//                         .iter()
//                         .filter_map(|&p| p.drafts.is_contain(v).then_some(p))
//                         .collect::<Vec<&Cell>>();
//                     (tmp.len() != 0).then_some((v, tmp))
//                 })
//                 .find_map(|(v, vp)| {
//                     {
//                         let vg = field.collect_all_drafts_cells_in_g(vp[0].gn.g);
//                         // 条件1：该列内的值都在同一个宫内
//                         let ret1 = !vp.iter().any(|&p| (p.gn.g != vp[0].gn.g));
//                         // 条件2：第一个值所在的宫，在其他列内有值
//                         let ret2 = vg
//                             .iter()
//                             .filter_map(|&vr_p_iter| {
//                                 ((vr_p_iter.rc.c != vp[0].rc.c) && vr_p_iter.drafts.is_contain(v))
//                                     .then_some(vr_p_iter)
//                             })
//                             .collect::<Vec<&Cell>>();
//                         if ret1 && ret2.len() != 0 {
//                             Some(Inference {
//                                 inference_type: InferenceType::LockedCandidatesInGridByCol,
//                                 condition: vp
//                                     .iter()
//                                     .map(|&p| TheCoordsAndTheValue {
//                                         the_cell: p,
//                                         the_value: v,
//                                     })
//                                     .collect::<Vec<TheCoordsAndTheValue>>(),
//                                 conclusion_set_value: None,
//                                 conclusion_remove_drafts: Some(
//                                     ret2.iter()
//                                         .map(|&p| TheCoordsAndTheValue {
//                                             the_cell: p,
//                                             the_value: v,
//                                         })
//                                         .collect::<Vec<TheCoordsAndTheValue>>(),
//                                 ),
//                             })
//                         } else {
//                             None
//                         }
//                     }
//                 })
//         })
// }

// // 显性数对排除法，在某一行中，存在二数对时，排除该行中其余数对草稿数
// pub fn search_naked_pair_in_row(field: &Field) {
//     field
//         .collect_all_drafts_cells_by_rc()
//         .iter()
//         .for_each(|vr| {
//             if vr.len() < 2 {
//                 // None
//             } else {
//                 for i in 0..vr.len() {
//                     let mut pair = None;
//                     if vr[i].drafts.to_vec().len() == 2 {
//                         for j in (i + 1)..vr.len() {
//                             if vr[j].drafts.delta_to(vr[i].drafts) == 0 {
//                                 pair = Some((vr[i], vr[j]));
//                             }
//                         }
//                     }
//                     if pair.is_none() {
//                         continue;
//                         // None
//                     } else {
//                         let (pair1, pair2) = pair.unwrap();
//                         let vec_pair = pair1.drafts.to_vec();
//                         vr.iter().for_each(|&p| {
//                             if p.drafts.is_contain(vec_pair[0]) {
//                                 println!("{:?}", p);
//                             }
//                             if p.drafts.is_contain(vec_pair[1]) {
//                                 println!("{:?}", p);
//                             }
//                         });
//                     }
//                 }
//                 // None
//             }
//         });
// }
