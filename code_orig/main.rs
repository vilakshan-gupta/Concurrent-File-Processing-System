use std::{
    fs::File,
    io::{self, BufRead, BufReader, Write},
    sync::{Arc, Mutex, Condvar},
    thread,
    path::Path,
    time::Instant,
};

struct LineReader {
    files: Vec<BufReader<File>>,
    current_file: usize,
    done: bool,
}

impl LineReader {
    fn new(file_paths: Vec<String>) -> io::Result<Self> {
        let files = file_paths
            .into_iter()
            .map(|path| File::open(path).map(BufReader::new))
            .collect::<Result<Vec<_>, io::Error>>()?;

        Ok(Self {
            files,
            current_file: 0,
            done: false,
        })
    }

    fn get_next_line(&mut self) -> Option<String> {
        if self.done {
            return None;
        }

        loop {
            if self.files.is_empty() {
                self.done = true;
                return None;
            }

            let file = &mut self.files[self.current_file];
            let mut line = String::new();
            if file.read_line(&mut line).unwrap_or(0) > 0 {
                self.current_file = (self.current_file + 1) % self.files.len();
                return Some(line);
            } else {
                self.files.remove(self.current_file);
                if self.current_file >= self.files.len() && !self.files.is_empty() {
                    self.current_file = 0;
                }
            }
        }
    }
}

fn write_lines(reader: Arc<(Mutex<LineReader>, Condvar)>, output_file: Arc<Mutex<File>>) {
    let (lock, cvar) = &*reader;
    let mut reader = lock.lock().unwrap();
    while !reader.done {
        if let Some(mut line) = reader.get_next_line() {
            line = line.to_string(); // Remove newline character
            {
                let mut output = output_file.lock().unwrap();
                write!(output, "{}", line).expect("Failed to write to output file");
            }
            cvar.notify_one();
        } else {
            break;
        }
    }
    cvar.notify_all(); // Notify any waiting threads that no more lines are available
}



fn get_files_in_directory(dir: &Path) -> io::Result<Vec<String>> {
    let mut files = Vec::new();
    for entry in dir.read_dir()? {
        let path = entry?.path();
        if path.is_file() {
            files.push(path.to_string_lossy().into_owned());
        }
    }
    Ok(files)
}

fn main() -> io::Result<()> {
    let input_dir = Path::new("./../tests");
    let output_file_path = "./../output_rust.txt";

    let file_paths = get_files_in_directory(input_dir)?;
    let output_file = Arc::new(Mutex::new(File::create(output_file_path)?));
    let reader = Arc::new((Mutex::new(LineReader::new(file_paths)?), Condvar::new()));

    // Start time measurement
    let start_time = Instant::now();

    let writer_thread = {
        let reader_clone = Arc::clone(&reader);
        let output_file_clone = Arc::clone(&output_file);
        thread::spawn(move || write_lines(reader_clone, output_file_clone))
    };

    writer_thread.join().expect("Writer thread panicked");

    // End time measurement
    let elapsed_time = start_time.elapsed();
    println!("Total Elapsed Time: {:?}", elapsed_time);

    Ok(())
}



