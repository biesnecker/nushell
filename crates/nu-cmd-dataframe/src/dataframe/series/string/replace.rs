use super::super::super::values::{Column, NuDataFrame};

use nu_engine::CallExt;
use nu_protocol::{
    ast::Call,
    engine::{Command, EngineState, Stack},
    Category, Example, PipelineData, ShellError, Signature, Span, SyntaxShape, Type, Value,
};
use polars::prelude::{IntoSeries, Utf8NameSpaceImpl};

#[derive(Clone)]
pub struct Replace;

impl Command for Replace {
    fn name(&self) -> &str {
        "dfr replace"
    }

    fn usage(&self) -> &str {
        "Replace the leftmost (sub)string by a regex pattern."
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .required_named(
                "pattern",
                SyntaxShape::String,
                "Regex pattern to be matched",
                Some('p'),
            )
            .required_named(
                "replace",
                SyntaxShape::String,
                "replacing string",
                Some('r'),
            )
            .input_output_type(
                Type::Custom("dataframe".into()),
                Type::Custom("dataframe".into()),
            )
            .category(Category::Custom("dataframe".into()))
    }

    fn examples(&self) -> Vec<Example> {
        vec![Example {
            description: "Replaces string",
            example: "[abc abc abc] | dfr into-df | dfr replace --pattern ab --replace AB",
            result: Some(
                NuDataFrame::try_from_columns(vec![Column::new(
                    "0".to_string(),
                    vec![
                        Value::test_string("ABc"),
                        Value::test_string("ABc"),
                        Value::test_string("ABc"),
                    ],
                )])
                .expect("simple df for test should not fail")
                .into_value(Span::test_data()),
            ),
        }]
    }

    fn run(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
        call: &Call,
        input: PipelineData,
    ) -> Result<PipelineData, ShellError> {
        command(engine_state, stack, call, input)
    }
}

fn command(
    engine_state: &EngineState,
    stack: &mut Stack,
    call: &Call,
    input: PipelineData,
) -> Result<PipelineData, ShellError> {
    let pattern: String = call
        .get_flag(engine_state, stack, "pattern")?
        .expect("required value");
    let replace: String = call
        .get_flag(engine_state, stack, "replace")?
        .expect("required value");

    let df = NuDataFrame::try_from_pipeline(input, call.head)?;
    let series = df.as_series(call.head)?;
    let chunked = series.utf8().map_err(|e| {
        ShellError::GenericError(
            "Error conversion to string".into(),
            e.to_string(),
            Some(call.head),
            None,
            Vec::new(),
        )
    })?;

    let mut res = chunked.replace(&pattern, &replace).map_err(|e| {
        ShellError::GenericError(
            "Error finding pattern other".into(),
            e.to_string(),
            Some(call.head),
            None,
            Vec::new(),
        )
    })?;

    res.rename(series.name());

    NuDataFrame::try_from_series(vec![res.into_series()], call.head)
        .map(|df| PipelineData::Value(NuDataFrame::into_value(df, call.head), None))
}

#[cfg(test)]
mod test {
    use super::super::super::super::test_dataframe::test_dataframe;
    use super::*;

    #[test]
    fn test_examples() {
        test_dataframe(vec![Box::new(Replace {})])
    }
}
