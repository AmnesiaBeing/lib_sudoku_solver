pub mod inferences;
pub mod types;

#[cfg(test)]
mod tests {
    use crate::{inferences::InferenceSet, types::Field};

    #[test]
    fn it_works() {
        println!("begin test.");

        let field1 = Field::initial_by_string(
            &"070009800008002006906100000600000150030801020092000008000003701800600300001900060"
                .to_string(),
        )
        .unwrap();

        // let field2 = Field::initial_by_string(
        //     &"615800790290600015040000260000080000730512046000090000080000030900008071071060582"
        //         .to_string(),
        // )
        // .unwrap();

        // let field3 = Field::initial_by_string(
        //     &"010076000805000300000000000270000000000500100600000000003000002000900040000000076"
        //         .to_string(),
        // )
        // .unwrap();

        // let field4 = Field::initial_by_string(
        //     &"800000000003600000070090200050007000000045700000100030001000068008500010090000400"
        //         .to_string(),
        // )
        // .unwrap();

        // let field5 = Field::initial_by_string(
        //     &"000000100000500306000000500030600412060300958800000000000000000100000000000000000"
        //         .to_string(),
        // )
        // .unwrap();

        // let field6 = Field::initial_by_string(
        //     &"060000000100000054000000700003000001008010070051000000080900000007100000010000000"
        //         .to_string(),
        // )
        // .unwrap();

        let field7 = Field::initial_by_string(
            &"586000020020465873437020516300710068008000100010082000073090045000000390090253080"
                .to_string(),
        )
        .unwrap();

        fn sovle(field: &Field) {
            let mut field = field.clone();
            field.print();
            let ifs = InferenceSet::new();
            loop {
                let inference = ifs.analyze(&field);
                match inference {
                    Some(inference) => {
                        println!("{:?}", inference);
                        field = InferenceSet::apply(&field, inference);
                        field.print();
                        if field.check_if_finish() {
                            println!("推导完毕!");
                            field.print();
                            // break;
                        }
                    }
                    None => {
                        println!("无法推导!");
                        // break;
                    }
                }
            }
        }

        sovle(&field1);
        // sovle(&field2);
        // sovle(&field3);
        // sovle(&field4);
        // sovle(&field5);
        // sovle(&field6);
        // sovle(&field7);

        fn generate_combinations(
            len: usize,
            size: usize,
            current: usize,
            path: &mut Vec<usize>,
            all_combinations: &mut Vec<(Vec<usize>, Vec<usize>)>,
        ) {
            if path.len() == len {
                return;
            }
            if path.len() == size {
                let mut remaining = Vec::new();
                for set in 0..len {
                    if !path.contains(&set) {
                        remaining.push(set);
                    }
                }
                all_combinations.push((path.clone(), remaining));
                return;
            }
            for i in current..len {
                path.push(i);
                generate_combinations(len, size, i + 1, path, all_combinations);
                path.pop();
            }
        }

        let mut all_combinations = Vec::new();
        for size in 2..=4 {
            let mut paths = Vec::new();
            println!("size: {:?}, paths: {:?}", size, paths);
            generate_combinations(3, size, 0, &mut paths, &mut all_combinations);
        }
        println!("{:?}", all_combinations);
    }
}
