pub mod inferences;
pub mod types;

#[cfg(test)]
mod tests {
    use std::{borrow::Borrow, result};

    use crate::{
        inferences::{InferenceResult, InferenceSet},
        types::Field,
    };

    #[test]
    fn it_works() {
        let field1 = Field::initial_by_string(
            &"070009800008002006906100000600000150030801020092000008000003701800600300001900060"
                .to_string(),
        )
        .unwrap();

        let field2 = Field::initial_by_string(
            &"615800790290600015040000260000080000730512046000090000080000030900008071071060582"
                .to_string(),
        )
        .unwrap();

        let field3 = Field::initial_by_string(
            &"010076000805000300000000000270000000000500100600000000003000002000900040000000076"
                .to_string(),
        )
        .unwrap();

        let field4 = Field::initial_by_string(
            &"800000000003600000070090200050007000000045700000100030001000068008500010090000400"
                .to_string(),
        )
        .unwrap();

        let field5 = Field::initial_by_string(
            &"000000100000500306000000500030600412060300958800000000000000000100000000000000000"
                .to_string(),
        )
        .unwrap();

        let field6 = Field::initial_by_string(
            &"060000000100000054000000700003000001008010070051000000080900000007100000010000000"
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

        sovle(&field1);
        sovle(&field2);
        sovle(&field3);
        sovle(&field4);
        sovle(&field5);
        sovle(&field6);
    }
}
