use crate::types::{Cell, CellStatus, CellValue, Field};

#[derive(Copy, Clone)]
pub struct CellAndValue<'a> {
    cell: &'a Cell,
    value: CellValue,
}

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
    OnlyOneRightInCol,
    OnlyOneRightInGrid,
    LockedCandidatesInRowByGrid,
    LockedCandidatesInColByGrid,
    LockedCandidatesInGridByRow,
    LockedCandidatesInGridByCol,
    NakedPairInRow,
    NakedPairInCol,
    NakedPairInGrid,
    NakedTripleInRow,
    NakedTripleInCol,
    NakedTripleInGrid,
    NakedQuadrupleInRow,
    NakedQuadrupleInCol,
    NakedQuadrupleInGrid,
}

pub struct Inferences;
impl Inferences {
    pub fn search<'a>(field: &'a Field) -> Option<Inference> {
        let vec_fn_inference: Vec<FnInference> = vec![
            search_only_one_left,
            search_only_one_right_in_row,
            search_only_one_right_in_col,
            search_only_one_right_in_grid,
            search_locked_candidates_in_row_col_by_grid,
            search_locked_candidates_in_grid_by_row,
            search_locked_candidates_in_grid_by_col,
        ];
        vec_fn_inference.iter().find_map(|&fn_t| fn_t(field))
    }

    pub fn apply(field: &Field, inference: Inference) -> Field {
        let mut ret = field.clone();
        if inference.conclusion_set_value.is_some() {
            inference
                .conclusion_set_value
                .unwrap()
                .iter()
                .for_each(|cv| {
                    let p = ret.get_cell_mut_by_rc(cv.cell.rc);
                    p.value = cv.value;
                    p.status = CellStatus::SOLVE;
                })
        };
        if inference.conclusion_remove_drafts.is_some() {
            inference
                .conclusion_remove_drafts
                .unwrap()
                .iter()
                .for_each(|cv| {
                    let p = ret.get_cell_mut_by_rc(cv.cell.rc);
                    p.drafts.remove_draft(cv.value);
                })
        }
        ret
    }
}

impl std::fmt::Debug for CellAndValue<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self.cell)
    }
}

impl std::fmt::Debug for Inference<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "因为 ")?;

        match self.inference_type {
            InferenceType::OnlyOneLeft => {
                write!(
                    f,
                    "{:?} 的 {:?} 在格内唯一",
                    self.condition[0].cell.rc, self.condition[0].value,
                )?;

                write!(f, "，推导出： ")?;

                write!(
                    f,
                    "{:?} 填写 {:?} ",
                    self.condition[0].cell.rc, self.condition[0].value
                )?;

                if self.conclusion_remove_drafts.is_some() {
                    write!(f, "，且移除 ")?;
                    self.conclusion_remove_drafts
                        .as_ref()
                        .unwrap()
                        .iter()
                        .for_each(|&cv| {
                            write!(f, "{:?} ", cv.cell.rc).unwrap();
                        });
                    write!(f, "的 {:?}", self.condition[0].value)?;
                }
                write!(f, "。")?;
            }
            InferenceType::OnlyOneRightInRow => {
                write!(
                    f,
                    "{:?} 的 {:?} 在行内唯一",
                    self.condition[0].cell.rc, self.condition[0].value
                )?;

                write!(f, "，推导出： ")?;

                write!(
                    f,
                    "{:?} 填写 {:?} ",
                    self.condition[0].cell.rc, self.condition[0].value
                )?;

                if self.conclusion_remove_drafts.is_some() {
                    write!(f, "，且移除 ")?;
                    self.conclusion_remove_drafts
                        .as_ref()
                        .unwrap()
                        .iter()
                        .for_each(|&cv| {
                            write!(f, "{:?} ", cv.cell.rc).unwrap();
                        });
                    write!(f, "的 {:?}", self.condition[0].value)?;
                }
                write!(f, "。")?;
            }
            InferenceType::OnlyOneRightInCol => {
                write!(
                    f,
                    "{:?} 的 {:?} 在列内唯一",
                    self.condition[0].cell.rc, self.condition[0].value
                )?;

                write!(f, "，推导出： ")?;

                write!(
                    f,
                    "{:?} 填写 {:?} ",
                    self.condition[0].cell.rc, self.condition[0].value
                )?;

                if self.conclusion_remove_drafts.is_some() {
                    write!(f, "，且移除 ")?;
                    self.conclusion_remove_drafts
                        .as_ref()
                        .unwrap()
                        .iter()
                        .for_each(|&cv| {
                            write!(f, "{:?} ", cv.cell.rc).unwrap();
                        });
                    write!(f, "的 {:?}", self.condition[0].value)?;
                }
                write!(f, "。")?;
            }
            InferenceType::OnlyOneRightInGrid => {
                write!(
                    f,
                    "{:?} 的 {:?} 在宫内唯一",
                    self.condition[0].cell.gn, self.condition[0].value
                )?;

                write!(f, "，推导出： ")?;

                write!(
                    f,
                    "{:?} 填写 {:?} ",
                    self.condition[0].cell.gn, self.condition[0].value
                )?;

                if self.conclusion_remove_drafts.is_some() {
                    write!(f, "，且移除 ")?;
                    self.conclusion_remove_drafts
                        .as_ref()
                        .unwrap()
                        .iter()
                        .for_each(|&cv| {
                            write!(f, "{:?} ", cv.cell.rc).unwrap();
                        });
                    write!(f, "的 {:?}", self.condition[0].value)?;
                }
                write!(f, "。")?;
            }
            InferenceType::LockedCandidatesInRowByGrid => {
                self.condition.iter().for_each(|&cv| {
                    write!(f, "{:?} ", cv.cell.rc).unwrap();
                });
                write!(
                    f,
                    "的 {:?} 在宫 G{:?} 内，只在 R{:?} 中存在，推导出： ",
                    self.condition[0].value,
                    self.condition[0].cell.gn.g + 1,
                    self.condition[0].cell.rc.r + 1
                )?;

                self.conclusion_remove_drafts
                    .as_ref()
                    .unwrap()
                    .iter()
                    .for_each(|&p| {
                        write!(f, "{:?} ", p.cell.rc).unwrap();
                    });
                write!(f, "不能填写 {:?} ，需要移除。", self.condition[0].value)?;
            }
            InferenceType::LockedCandidatesInColByGrid => {
                self.condition.iter().for_each(|&cv| {
                    write!(f, "{:?} ", cv.cell.gn,).unwrap();
                });
                write!(
                    f,
                    "的 {:?} 在宫 G{:?} 内，只在 C{:?} 中存在，推导出： ",
                    self.condition[0].value,
                    self.condition[0].cell.gn.g + 1,
                    self.condition[0].cell.rc.c + 1
                )?;

                self.conclusion_remove_drafts
                    .as_ref()
                    .unwrap()
                    .iter()
                    .for_each(|&p| {
                        write!(f, "{:?} ", p.cell.rc).unwrap();
                    });
                write!(f, "不能填写 {:?} ，需要移除。", self.condition[0].value)?;
            }
            InferenceType::LockedCandidatesInGridByRow => {
                self.condition.iter().for_each(|&cv| {
                    write!(f, "{:?} ", cv.cell.rc).unwrap();
                });
                write!(
                    f,
                    "的 {:?} 在 R{:?} 内，只在宫 G{:?} 中存在，推导出： ",
                    self.condition[0].value,
                    self.condition[0].cell.rc.r + 1,
                    self.condition[0].cell.gn.g + 1
                )?;

                self.conclusion_remove_drafts
                    .as_ref()
                    .unwrap()
                    .iter()
                    .for_each(|&p| {
                        write!(f, "{:?} ", p.cell.gn).unwrap();
                    });
                write!(f, "不能填写 {:?} ，需要移除。", self.condition[0].value)?;
            }
            InferenceType::LockedCandidatesInGridByCol => {
                self.condition.iter().for_each(|&cv| {
                    write!(f, "{:?} ", cv.cell.rc).unwrap();
                });
                write!(
                    f,
                    "的 {:?} 在 C{:?} 内，只在宫 G{:?} 中存在，推导出： ",
                    self.condition[0].value,
                    self.condition[0].cell.rc.c + 1,
                    self.condition[0].cell.gn.g + 1
                )?;

                self.conclusion_remove_drafts
                    .as_ref()
                    .unwrap()
                    .iter()
                    .for_each(|&p| {
                        write!(f, "{:?} ", p.cell.gn).unwrap();
                    });
                write!(f, "不能填写 {:?} ，需要移除。", self.condition[0].value)?;
            }
            InferenceType::NakedPairInRow => todo!(),
            InferenceType::NakedPairInCol => todo!(),
            InferenceType::NakedPairInGrid => todo!(),
            InferenceType::NakedTripleInRow => todo!(),
            InferenceType::NakedTripleInCol => todo!(),
            InferenceType::NakedTripleInGrid => todo!(),
            InferenceType::NakedQuadrupleInRow => todo!(),
            InferenceType::NakedQuadrupleInCol => todo!(),
            InferenceType::NakedQuadrupleInGrid => todo!(),
        }

        write!(f, "")
    }
}

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
    field
        .collect_all_drafts_cells_by_rc()
        .iter()
        .find_map(|vr| {
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
                        let cv = CellAndValue {
                            cell: p,
                            value: ret,
                        };
                        Some(Inference {
                            inference_type: InferenceType::OnlyOneRightInRow,
                            condition: vec![cv],
                            conclusion_set_value: Some(vec![cv]),
                            conclusion_remove_drafts: make_removing_drafts_when_set_value(
                                field, cv,
                            ),
                        })
                    })
            })
        })
}

// 按列排除法，每行中如果存在唯一草稿值，则填写该值，同时去除其余同一列宫的草稿值
fn search_only_one_right_in_col<'a>(field: &'a Field) -> Option<Inference> {
    field
        .collect_all_drafts_cells_by_cr()
        .iter()
        .find_map(|vc| {
            vc.iter().find_map(|&p| {
                p.drafts
                    .to_vec()
                    .iter()
                    .find(|&v| {
                        !(vc.iter().fold(false, |acc, p_iter| {
                            acc || ((p_iter.rc.r != p.rc.r) && (p_iter.drafts.is_contain(*v)))
                        }))
                    })
                    .and_then(|&ret| {
                        let cv = CellAndValue {
                            cell: p,
                            value: ret,
                        };
                        Some(Inference {
                            inference_type: InferenceType::OnlyOneRightInCol,
                            condition: vec![cv],
                            conclusion_set_value: Some(vec![cv]),
                            conclusion_remove_drafts: make_removing_drafts_when_set_value(
                                field, cv,
                            ),
                        })
                    })
            })
        })
}

// 按宫排除法，每行中如果存在唯一草稿值，则填写该值，同时去除其余同一列宫的草稿值
fn search_only_one_right_in_grid<'a>(field: &'a Field) -> Option<Inference> {
    field
        .collect_all_drafts_cells_by_gn()
        .iter()
        .find_map(|vg| {
            vg.iter().find_map(|&p| {
                p.drafts
                    .to_vec()
                    .iter()
                    .find(|&v| {
                        !(vg.iter().fold(false, |acc, p_iter| {
                            acc || ((p_iter.gn.n != p.gn.n) && (p_iter.drafts.is_contain(*v)))
                        }))
                    })
                    .and_then(|&ret| {
                        let cv = CellAndValue {
                            cell: p,
                            value: ret,
                        };
                        Some(Inference {
                            inference_type: InferenceType::OnlyOneRightInGrid,
                            condition: vec![cv],
                            conclusion_set_value: Some(vec![cv]),
                            conclusion_remove_drafts: make_removing_drafts_when_set_value(
                                field, cv,
                            ),
                        })
                    })
            })
        })
}

// 当一宫内的某种草稿值当且仅当在同一行/列时，可以排除行/列内其余格子的该草稿值
pub fn search_locked_candidates_in_row_col_by_grid<'a>(field: &'a Field) -> Option<Inference> {
    field
        .collect_all_drafts_cells_by_gn()
        .iter()
        .find_map(|vg| {
            CellValue::vec_for_iter()
                .iter()
                .filter_map(|&v| {
                    let tmp: Vec<&Cell> = vg
                        .iter()
                        .filter_map(|&p| p.drafts.is_contain(v).then_some(p))
                        .collect::<Vec<&Cell>>();
                    (tmp.len() != 0).then_some((v, tmp))
                })
                .find_map(|(v, vp)| {
                    {
                        let vr = field.collect_all_drafts_cells_in_r(vp[0].rc.r);
                        // 条件1：该宫内其他行没有这个值
                        let ret1 = !vp.iter().any(|&p| (p.rc.r != vp[0].rc.r));
                        // 条件2：宫外该行内有这个值
                        let ret2 = vr
                            .iter()
                            .filter_map(|&vr_p_iter| {
                                ((vr_p_iter.gn.g != vp[0].gn.g) && vr_p_iter.drafts.is_contain(v))
                                    .then_some(vr_p_iter)
                            })
                            .collect::<Vec<&Cell>>();
                        if ret1 && ret2.len() != 0 {
                            Some(Inference {
                                inference_type: InferenceType::LockedCandidatesInRowByGrid,
                                condition: vp
                                    .iter()
                                    .map(|&p| CellAndValue { cell: p, value: v })
                                    .collect::<Vec<CellAndValue>>(),
                                conclusion_set_value: None,
                                conclusion_remove_drafts: Some(
                                    ret2.iter()
                                        .map(|&p| CellAndValue { cell: p, value: v })
                                        .collect::<Vec<CellAndValue>>(),
                                ),
                            })
                        } else {
                            None
                        }
                    }
                    .or({
                        {
                            let vc = field.collect_all_drafts_cells_in_c(vp[0].rc.c);
                            // 条件1：该宫内其他列没有这个值
                            let ret1 = !vp.iter().any(|&p| (p.rc.c != vp[0].rc.c));
                            // 条件2：宫外该列内有这个值
                            let ret2 = vc
                                .iter()
                                .filter_map(|&vc_p_iter| {
                                    ((vc_p_iter.gn.g != vp[0].gn.g)
                                        && vc_p_iter.drafts.is_contain(v))
                                    .then_some(vc_p_iter)
                                })
                                .collect::<Vec<&Cell>>();
                            (ret1 && ret2.len() != 0).then_some(Inference {
                                inference_type: InferenceType::LockedCandidatesInColByGrid,
                                condition: vp
                                    .iter()
                                    .map(|&p| CellAndValue { cell: p, value: v })
                                    .collect::<Vec<CellAndValue>>(),
                                conclusion_set_value: None,
                                conclusion_remove_drafts: Some(
                                    ret2.iter()
                                        .map(|&p| CellAndValue { cell: p, value: v })
                                        .collect::<Vec<CellAndValue>>(),
                                ),
                            })
                        }
                    })
                })
        })
}

// 当一行的草稿数正好在一宫时，排除该宫的其他草稿数
pub fn search_locked_candidates_in_grid_by_row<'a>(field: &'a Field) -> Option<Inference> {
    field
        .collect_all_drafts_cells_by_rc()
        .iter()
        .find_map(|vr| {
            CellValue::vec_for_iter()
                .iter()
                .filter_map(|&v| {
                    let tmp: Vec<&Cell> = vr
                        .iter()
                        .filter_map(|&p| p.drafts.is_contain(v).then_some(p))
                        .collect::<Vec<&Cell>>();
                    (tmp.len() != 0).then_some((v, tmp))
                })
                .find_map(|(v, vp)| {
                    {
                        let vg = field.collect_all_drafts_cells_in_g(vp[0].gn.g);
                        // 条件1：该行内的值都在同一个宫内
                        let ret1 = !vp.iter().any(|&p| (p.gn.g != vp[0].gn.g));
                        // 条件2：第一个值所在的宫，在其他行内有值
                        let ret2 = vg
                            .iter()
                            .filter_map(|&vr_p_iter| {
                                ((vr_p_iter.rc.r != vp[0].rc.r) && vr_p_iter.drafts.is_contain(v))
                                    .then_some(vr_p_iter)
                            })
                            .collect::<Vec<&Cell>>();
                        if ret1 && ret2.len() != 0 {
                            Some(Inference {
                                inference_type: InferenceType::LockedCandidatesInGridByRow,
                                condition: vp
                                    .iter()
                                    .map(|&p| CellAndValue { cell: p, value: v })
                                    .collect::<Vec<CellAndValue>>(),
                                conclusion_set_value: None,
                                conclusion_remove_drafts: Some(
                                    ret2.iter()
                                        .map(|&p| CellAndValue { cell: p, value: v })
                                        .collect::<Vec<CellAndValue>>(),
                                ),
                            })
                        } else {
                            None
                        }
                    }
                })
        })
}

// 当一列的草稿数正好在一宫时，排除该宫的其他草稿数
pub fn search_locked_candidates_in_grid_by_col<'a>(field: &'a Field) -> Option<Inference> {
    field
        .collect_all_drafts_cells_by_cr()
        .iter()
        .find_map(|vc| {
            CellValue::vec_for_iter()
                .iter()
                .filter_map(|&v| {
                    let tmp: Vec<&Cell> = vc
                        .iter()
                        .filter_map(|&p| p.drafts.is_contain(v).then_some(p))
                        .collect::<Vec<&Cell>>();
                    (tmp.len() != 0).then_some((v, tmp))
                })
                .find_map(|(v, vp)| {
                    {
                        let vg = field.collect_all_drafts_cells_in_g(vp[0].gn.g);
                        // 条件1：该列内的值都在同一个宫内
                        let ret1 = !vp.iter().any(|&p| (p.gn.g != vp[0].gn.g));
                        // 条件2：第一个值所在的宫，在其他列内有值
                        let ret2 = vg
                            .iter()
                            .filter_map(|&vr_p_iter| {
                                ((vr_p_iter.rc.c != vp[0].rc.c) && vr_p_iter.drafts.is_contain(v))
                                    .then_some(vr_p_iter)
                            })
                            .collect::<Vec<&Cell>>();
                        if ret1 && ret2.len() != 0 {
                            Some(Inference {
                                inference_type: InferenceType::LockedCandidatesInGridByCol,
                                condition: vp
                                    .iter()
                                    .map(|&p| CellAndValue { cell: p, value: v })
                                    .collect::<Vec<CellAndValue>>(),
                                conclusion_set_value: None,
                                conclusion_remove_drafts: Some(
                                    ret2.iter()
                                        .map(|&p| CellAndValue { cell: p, value: v })
                                        .collect::<Vec<CellAndValue>>(),
                                ),
                            })
                        } else {
                            None
                        }
                    }
                })
        })
}

// 显性数对排除法，在某一行中，存在二数对时，排除该行中其余数对草稿数
pub fn search_naked_pair_in_row<'a>(field: &'a Field) {
    field
        .collect_all_drafts_cells_by_rc()
        .iter()
        .for_each(|vr| {
            if vr.len() < 2 {
                // None
            } else {
                for i in 0..vr.len() {
                    let mut pair = None;
                    if vr[i].drafts.to_vec().len() == 2 {
                        for j in (i + 1)..vr.len() {
                            if vr[j].drafts.delta_to(vr[i].drafts) == 0 {
                                pair = Some((vr[i], vr[j]));
                            }
                        }
                    }
                    if pair.is_none() {
                        continue;
                        // None
                    } else {
                        let (pair1, pair2) = pair.unwrap();
                        let vec_pair = pair1.drafts.to_vec();
                        vr.iter().for_each(|&p| {
                            if p.drafts.is_contain(vec_pair[0]) {
                                println!("{:?}", p);
                            }
                            if p.drafts.is_contain(vec_pair[1]) {
                                println!("{:?}", p);
                            }
                        });
                    }
                }
                // None
            }
        });
}
