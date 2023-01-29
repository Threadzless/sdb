
#[macro_export]
macro_rules! parse_optional_clauses {
    (
        ($input: ident, $valid: ident) => {
            $([ $($kw: literal),+ ] =( $idx: literal )=> $do: expr ),+
        }
    ) => {
        loop {
            if $input.is_empty() {
                break;
            }

            let next_keyword = $input.cursor()
                .ident()
                .map(|(i, _)| i.to_string().to_ascii_uppercase());

            let mut prev_idx = -1;

            match next_keyword.as_deref() {
                $(
                    $( Option::Some($kw) )|+ => {
                        // prev_idx = $idx;
                        $do
                    }
                ),+
                _ => {
                    ::proc_macro_error::emit_error!(
                        $input.span(), "No Clause???";
                        help = "{}", $valid;
                    )
                },
            }
        }
    };
}
