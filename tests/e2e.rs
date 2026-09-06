use std::fs;
use std::path::Path;
use std::process::Command;

macro_rules! e2e_test {
    ($test_name:ident, $source_file:expr, $expected_output:expr) => {
        #[test]
        fn $test_name() {
            let source_path = Path::new($source_file);
            let output_bin = Path::new("hello_world");
            let output_dir = Path::new("./output");

            assert!(source_path.exists(), "Source file not found");

            let compile_result = Command::new("cargo")
                .arg("run")
                .arg("-q")
                .arg("--")
                .arg(source_path)
                .arg("-o")
                .arg(output_bin)
                .output()
                .expect("Cargo could not started");

            assert!(
                compile_result.status.success(),
                "Compilation failed!\nStderr:\n{}",
                String::from_utf8_lossy(&compile_result.stderr)
            );

            let run_result = Command::new(output_dir.join(output_bin))
                .output()
                .expect("Compiled binary could not be started");

            assert!(
                run_result.status.success(),
                "Error while executing the binary"
            );

            let actual_output = String::from_utf8_lossy(&run_result.stdout);
            let expected_output = $expected_output;

            assert_eq!(
                actual_output.trim(),
                expected_output.trim(),
                "Console output does not match"
            );

            let _ = fs::remove_file(output_bin);
        }
    };
}

e2e_test!(
    hello_world_e2e,
    "./examples/hello_world.ba",
    "Hello World\n"
);
e2e_test!(fibonacci_e2e, "./examples/fibonacci.ba", "55");
e2e_test!(ggt_e2e, "./examples/ggt.ba", "6");
e2e_test!(point_e2e, "./examples/point.ba", "37.400000");
e2e_test!(power_e2e, "./examples/power.ba", "16");
e2e_test!(sum_e2e, "./examples/sum.ba", "166.500000");
e2e_test!(
    test_foobar,
    "examples/foobar.ba",
    "1\n2\nFoo\n4\nBar\nFoo\n7\n8\nFoo\nBar\n11\nFoo\n13\n14\nFooBar\n16\n17\nFoo\n19\nBar\n"
);
