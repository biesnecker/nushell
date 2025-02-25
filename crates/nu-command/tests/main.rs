use nu_protocol::{
    engine::{EngineState, StateWorkingSet},
    Category, Span,
};
use quickcheck_macros::quickcheck;

mod commands;
mod format_conversions;

fn create_default_context() -> EngineState {
    nu_command::add_shell_command_context(nu_cmd_lang::create_default_context())
}

#[quickcheck]
fn quickcheck_parse(data: String) -> bool {
    let (tokens, err) = nu_parser::lex(data.as_bytes(), 0, b"", b"", true);

    if err.is_none() {
        let context = create_default_context();
        {
            let mut working_set = StateWorkingSet::new(&context);
            let _ = working_set.add_file("quickcheck".into(), data.as_bytes());

            let _ =
                nu_parser::parse_block(&mut working_set, &tokens, Span::new(0, 0), false, false);
        }
    }
    true
}

#[test]
fn signature_name_matches_command_name() {
    let ctx = create_default_context();
    let decls = ctx.get_decls_sorted(true);
    let mut failures = Vec::new();

    for (name_bytes, decl_id) in decls {
        let cmd = ctx.get_decl(decl_id);
        let cmd_name = String::from_utf8_lossy(&name_bytes);
        let sig_name = cmd.signature().name;
        let category = cmd.signature().category;

        if cmd_name != sig_name {
            failures.push(format!(
                "{cmd_name} ({category:?}): Signature name \"{sig_name}\" is not equal to the command name \"{cmd_name}\""
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "Name mismatch:\n{}",
        failures.join("\n")
    );
}

#[test]
fn commands_declare_input_output_types() {
    let ctx = create_default_context();
    let decls = ctx.get_decls_sorted(true);
    let mut failures = Vec::new();

    for (_, decl_id) in decls {
        let cmd = ctx.get_decl(decl_id);
        let sig_name = cmd.signature().name;
        let category = cmd.signature().category;

        if matches!(category, Category::Removed | Category::Custom(_)) {
            // Deprecated/Removed commands don't have to conform
            // TODO: also upgrade the `--features dataframe` commands
            continue;
        }

        if cmd.signature().input_output_types.is_empty() {
            failures.push(format!(
                "{sig_name} ({category:?}): No pipeline input/output type signatures found"
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "Command missing type annotations:\n{}",
        failures.join("\n")
    );
}

#[test]
fn no_search_term_duplicates() {
    let ctx = crate::create_default_context();
    let decls = ctx.get_decls_sorted(true);
    let mut failures = Vec::new();

    for (name_bytes, decl_id) in decls {
        let cmd = ctx.get_decl(decl_id);
        let cmd_name = String::from_utf8_lossy(&name_bytes);
        let search_terms = cmd.search_terms();
        let category = cmd.signature().category;

        for search_term in search_terms {
            if cmd_name.contains(search_term) {
                failures.push(format!("{cmd_name} ({category:?}): Search term \"{search_term}\" is substring of command name \"{cmd_name}\""));
            }
        }
    }

    assert!(
        failures.is_empty(),
        "Duplication in search terms:\n{}",
        failures.join("\n")
    );
}
