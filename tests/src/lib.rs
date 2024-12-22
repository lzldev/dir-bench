#![feature(test)]

#[cfg(test)]
mod benchs {
    extern crate test;

    use dir_bench::{Fixture, dir_bench};
    use test::Bencher;

    #[dir_bench(
        dir: "$CARGO_MANIFEST_DIR/../fixtures/",
        glob: "**/*.txt"
        loader: std::fs::read_to_string
    )]
    fn word_counter(bench: &mut Bencher, fixture: Fixture<std::io::Result<String>>) {
        let ct = fixture.into_content().unwrap();

        bench.iter(|| {
            let all_lines = ct
                .lines()
                .map(|v| v.split(" ").collect::<Vec<_>>())
                .collect::<Vec<_>>();

            let count_lines = all_lines.len();
            let count_words = all_lines.iter().map(|v| v.len()).sum::<usize>();
            let medium = count_words / count_lines;

            println!("lines:{}", medium);
            println!("words:{}", medium);
            println!("words/line:{}", medium);
        })
    }

    #[dir_bench(
        dir: "$CARGO_MANIFEST_DIR/../fixtures/",
        glob: "**/*.txt"
    )]
    fn word_counter_reduce(bench: &mut Bencher, fixture: Fixture<&str>) {
        let ct = fixture.into_content();

        bench.iter(|| {
            let all_lines = ct
                .lines()
                .map(|v| v.split(" ").collect::<Vec<_>>())
                .collect::<Vec<_>>();

            let count_lines = all_lines.len();
            let count_words = all_lines.iter().fold(0usize, |c, v| c + v.len());
            let medium = count_words / count_lines;

            println!("lines:{}", count_lines);
            println!("words:{}", count_words);
            println!("words/line:{}\n\n", medium);
        })
    }
}
