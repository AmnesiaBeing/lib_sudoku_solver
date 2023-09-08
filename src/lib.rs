pub mod inferences;
pub mod types;

#[cfg(test)]
mod tests {
    use crate::{inferences::Inferences, types::Field};

    #[test]
    fn it_works() {
        // 简单谜题：070009800008002006906100000600000150030801020092000008000003701800600300001900060
        // 随机谜题：100020974040000000008040100000086000680075000000010008030062540000050000485000000
        // 简单17数：010076000805000300000000000270000000000500100600000000003000002000900040000000076
        // 复杂17数：800000000003600000070090200050007000000045700000100030001000068008500010090000400
        // 复杂17数：000000100000500306000000500030600412060300958800000000000000000100000000000000000
        // 复杂17数：060000000100000054000000700003000001008010070051000000080900000007100000010000000
        let mut field = Field::initial_by_string(
            &"070009800008002006906100000600000150030801020092000008000003701800600300001900060"
                .to_string(),
        )
        .unwrap();
        field.print();
        // println!("{:?}", field.find_conflict());
        // println!("{:?}", field.find_empty_drafts());

        loop {
            let inference = Inferences::search(&field);
            match inference {
                Some(inference) => {
                    println!("{:?}", inference);
                    field = Inferences::apply(&field, inference);
                    field.print();
                    if field.check_if_finish() {
                        println!("推倒完毕!");
                        break;
                    }
                }
                None => {
                    println!("无法推导!");
                    break;
                }
            }
        }
    }

    #[test]
    fn search_locked_candidates_in_row_col_by_grid_test() {
        let mut field = Field::initial_by_string(
            &"060000000100000054000000700003000001008010070051000000080900000007100000010000000"
                .to_string(),
        )
        .unwrap();
        field.print();
        println!(
            "{:?}",
            crate::inferences::search_locked_candidates_in_row_col_by_grid(&field)
        );
    }
}
