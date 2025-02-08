#[cfg(test)]
mod tests {
    use std::{
        // fs::{DirEntry, read_dir, read_to_string},
        path::Path,
        process::Command,
    };

    // #[test]
    // fn execute_tests() {
    //     let cases = read_dir("./cases").unwrap();

    //     let mut errors = vec![];
    //     for case in cases {
    //         let case = case.unwrap();

    //         match run_test(case) {
    //             Ok(_) => (),
    //             Err(msg) => errors.push(msg),
    //         }
    //     }

    //     if errors.len() > 0 {
    //         panic!("Errors:\n\n{}", errors.join("\n\n"));
    //     }
    // }

    // fn run_test(file: DirEntry) -> Result<(), String> {
    //     let contents = read_to_string(file.path()).unwrap();
    //     let lines = contents.split("\n").collect::<Vec<&str>>();

    //     let mut test_code: Vec<&str> = vec![];

    //     let mut idx = 0;
    //     for (i, line) in lines.iter().enumerate() {
    //         if line.starts_with("--- Test") {
    //             continue;
    //         }
    //         if line.starts_with("--- Expected") {
    //             idx = i;
    //             break;
    //         }

    //         test_code.push(line);
    //     }

    //     let mut expected_output: Vec<&str> = vec![];

    //     for line in &lines[idx + 1..] {
    //         if line.len() > 0 {
    //             expected_output.push(line);
    //         }
    //     }

    //     let input = test_code.join("\n");
    //     let output = Command::new("cargo")
    //         .arg("run")
    //         .arg("--")
    //         .arg("e")
    //         .arg(input)
    //         .output()
    //         .unwrap();
    //     let lines: Vec<&str> = std::str::from_utf8(&output.stdout)
    //         .unwrap()
    //         .split("\n")
    //         .collect::<Vec<&str>>();

    //     if !(lines.len() == expected_output.len() || lines.len() == expected_output.len() + 1) {
    //         return Err(format!(
    //             "{}: output length does not match expected output: {} != {}",
    //             file.file_name().to_string_lossy(),
    //             lines.len(),
    //             expected_output.len()
    //         ));
    //     }

    //     assert!(lines.len() == expected_output.len() || lines.len() == expected_output.len() + 1);
    //     for (i, expected) in expected_output.iter().enumerate() {
    //         if lines[i] != *expected {
    //             return Err(format!(
    //                 "{}: {} != {}\nFull outpu:\n{}",
    //                 file.file_name().to_string_lossy(),
    //                 lines[i],
    //                 expected,
    //                 expected_output.join("\n")
    //             ));
    //         }
    //     }

    //     Ok(())
    // }

    #[test]
    fn interpret_block() {
        let path = Path::new("cases/block.rl");
        let output = Command::new("cargo")
            .args(["run", "-p", "rl", "--", &path.display().to_string()])
            .output()
            .unwrap();

        let lines = std::str::from_utf8(output.stdout.as_slice())
            .unwrap()
            .split('\n')
            .collect::<Vec<&str>>();

        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "3");
        assert_eq!(lines[0], "3");
    }

    #[test]
    fn interpret_while() {
        let path = Path::new("cases/while.rl");
        let output = Command::new("cargo")
            .args(["run", "-p", "rl", "--", &path.display().to_string()])
            .output()
            .unwrap();

        let lines = std::str::from_utf8(output.stdout.as_slice())
            .unwrap()
            .split('\n')
            .collect::<Vec<&str>>();

        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "1");
        assert_eq!(lines[1], "0");
    }

    #[test]
    fn interpret_while_math() {
        let path = Path::new("cases/while_math.rl");
        let output = Command::new("cargo")
            .args(["run", "-p", "rl", "--", &path.display().to_string()])
            .output()
            .unwrap();

        let lines = std::str::from_utf8(output.stdout.as_slice())
            .unwrap()
            .split('\n')
            .collect::<Vec<&str>>();

        assert_eq!(lines.len(), 11);
        assert_eq!(lines[0], "10");
        assert_eq!(lines[1], "90");
        assert_eq!(lines[2], "720");
        assert_eq!(lines[3], "5040");
        assert_eq!(lines[4], "30240");
        assert_eq!(lines[5], "151200");
        assert_eq!(lines[6], "604800");
        assert_eq!(lines[7], "1814400");
        assert_eq!(lines[8], "3628800");
        assert_eq!(lines[9], "3628800");
    }

    #[test]
    fn interpret_for_loop() {
        let path = Path::new("cases/for_loop.rl");
        let output = Command::new("cargo")
            .args(["run", "-p", "rl", "--", &path.display().to_string()])
            .output()
            .unwrap();

        let lines = std::str::from_utf8(output.stdout.as_slice())
            .unwrap()
            .split('\n')
            .collect::<Vec<&str>>();

        let mut fibo = vec![];
        let mut a = 0;
        let mut b = 1;
        let mut temp;
        for _ in 0..21 {
            fibo.push(a);
            temp = b;
            b = a + b;
            a = temp;
        }

        assert_eq!(lines.len(), fibo.len() + 1);
        for i in 0..fibo.len() {
            assert_eq!(lines[i], fibo[i].to_string());
        }
    }

    #[test]
    fn interpret_fn_def() {
        let path = Path::new("cases/fn_def.rl");
        let output = Command::new("cargo")
            .args(["run", "-p", "rl", "--", &path.display().to_string()])
            .output()
            .unwrap();

        let lines = std::str::from_utf8(output.stdout.as_slice())
            .unwrap()
            .split('\n')
            .collect::<Vec<&str>>();

        assert_eq!(lines.len(), 4, "Output: '{}'", lines.join("\n"));
        assert_eq!(lines[0], "1");
        assert_eq!(lines[1], "2");
        assert_eq!(lines[2], "3");
    }

    #[test]
    fn interpret_fn_mod_local() {
        let path = Path::new("cases/fn_env.rl");
        let output = Command::new("cargo")
            .args(["run", "-p", "rl", "--", &path.display().to_string()])
            .output()
            .unwrap();

        let lines = std::str::from_utf8(output.stdout.as_slice())
            .unwrap()
            .split('\n')
            .collect::<Vec<&str>>();

        assert_eq!(lines.len(), 2, "Output: '{}'", lines.join("\n"));
        assert_eq!(lines[0], "3");
    }

    #[test]
    fn interpret_fn_return() {
        let path = Path::new("cases/fn_return.rl");
        let output = Command::new("cargo")
            .args(["run", "-p", "rl", "--", &path.display().to_string()])
            .output()
            .unwrap();

        let lines = std::str::from_utf8(output.stdout.as_slice())
            .unwrap()
            .split('\n')
            .collect::<Vec<&str>>();

        assert_eq!(lines.len(), 2, "Output: '{}'", lines.join("\n"));
        assert_eq!(lines[0], "5");
    }

    #[test]
    fn interpret_fn_no_return() {
        let path = Path::new("cases/fn_return_nil.rl");
        let output = Command::new("cargo")
            .args(["run", "-p", "rl", "--", &path.display().to_string()])
            .output()
            .unwrap();

        let lines = std::str::from_utf8(output.stdout.as_slice())
            .unwrap()
            .split('\n')
            .collect::<Vec<&str>>();

        assert_eq!(lines.len(), 4, "Output: '{}'", lines.join("\n"));
        assert_eq!(lines[0], "1");
        assert_eq!(lines[1], "2");
        assert_eq!(lines[2], "nil");
    }

    #[test]
    fn interpret_fn_cond_return() {
        let path = Path::new("cases/fn_cond_return.rl");
        let output = Command::new("cargo")
            .args(["run", "-p", "rl", "--", &path.display().to_string()])
            .output()
            .unwrap();

        let lines = std::str::from_utf8(output.stdout.as_slice())
            .unwrap()
            .split('\n')
            .collect::<Vec<&str>>();

        assert_eq!(lines.len(), 5, "Output: '{}'", lines.join("\n"));
        assert_eq!(lines[0], "3");
        assert_eq!(lines[1], "2");
        assert_eq!(lines[2], "1");
        assert_eq!(lines[3], "0");
    }

    #[test]
    fn interpret_fn_nested() {
        let path = Path::new("cases/fn_nested.rl");
        let output = Command::new("cargo")
            .args(["run", "-p", "rl", "--", &path.display().to_string()])
            .output()
            .unwrap();

        let lines = std::str::from_utf8(output.stdout.as_slice())
            .unwrap()
            .split('\n')
            .collect::<Vec<&str>>();

        assert_eq!(lines.len(), 3, "Output: '{}'", lines.join("\n"));
        assert_eq!(lines[0], "2");
        assert_eq!(lines[1], "3");
    }

    #[test]
    fn interpret_fn_closure() {
        let path = Path::new("cases/fn_closure.rl");
        let output = Command::new("cargo")
            .args(["run", "-p", "rl", "--", &path.display().to_string()])
            .output()
            .unwrap();

        let lines = std::str::from_utf8(output.stdout.as_slice())
            .unwrap()
            .split('\n')
            .collect::<Vec<&str>>();

        assert_eq!(lines.len(), 5, "Output: '{}'", lines.join("\n"));
        assert_eq!(lines[0], "1");
        assert_eq!(lines[1], "2");
        assert_eq!(lines[2], "1");
        assert_eq!(lines[3], "2");
    }

    #[test]
    fn interpret_fn_anon() {
        let path = Path::new("cases/fn_anon.rl");
        let output = Command::new("cargo")
            .args(["run", "-p", "rl", "--", &path.display().to_string()])
            .output()
            .unwrap();

        let lines = std::str::from_utf8(output.stdout.as_slice())
            .unwrap()
            .split('\n')
            .collect::<Vec<&str>>();

        assert_eq!(lines.len(), 4, "Output: '{}'", lines.join("\n"));
        assert_eq!(lines[0], "1");
        assert_eq!(lines[1], "2");
        assert_eq!(lines[2], "3");
    }

    #[test]
    fn interpret_fn_anon2() {
        let path = Path::new("cases/fn_anon2.rl");
        let output = Command::new("cargo")
            .args(["run", "-p", "rl", "--", &path.display().to_string()])
            .output()
            .unwrap();

        let lines = std::str::from_utf8(output.stdout.as_slice())
            .unwrap()
            .split('\n')
            .collect::<Vec<&str>>();

        assert_eq!(lines.len(), 2, "Output: '{}'", lines.join("\n"));
        assert_eq!(lines[0], "1");
    }
}
