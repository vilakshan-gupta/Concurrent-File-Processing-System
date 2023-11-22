#include <iostream>
#include <fstream>
#include <string>
#include <vector>
#include <thread>
#include <mutex>
#include <condition_variable>
#include <filesystem>

class LineReader {
public:
    LineReader(const std::vector<std::string>& file_paths)
        : file_paths(file_paths), current_file(0) {
        for (const auto& path : file_paths) {
            std::ifstream file(path);
            files.push_back(std::move(file));
        }
    }

    std::pair<bool, std::string> get_next_line() {
        std::unique_lock<std::mutex> lock(mtx);
        cv.wait(lock, [this] { return current_file < files.size(); });

        if (getline(files[current_file], line)) {
            if (++current_file >= files.size()) {
                current_file = 0;
            }
            cv.notify_all();
            return { true, line };
        }

        current_file++;
        if (current_file >= files.size()) {
            all_files_done = true;
        }
        cv.notify_all();
        return { false, "" };
    }

    bool is_done() const {
        return all_files_done;
    }

private:
    std::vector<std::ifstream> files;
    std::vector<std::string> file_paths;
    mutable std::mutex mtx;
    mutable std::condition_variable cv;
    size_t current_file;
    std::string line;
    bool all_files_done = false;
};

void write_lines(LineReader& reader, std::ofstream& output_file) {
    while (!reader.is_done()) {
        auto [success, line] = reader.get_next_line();
        if (success) {
            output_file << line << std::endl;
        }
    }
}

std::vector<std::string> get_files_in_directory(const std::string& directory) {
    std::vector<std::string> files;
    for (const auto& entry : std::filesystem::directory_iterator(directory)) {
        if (entry.is_regular_file()) {
            files.push_back(entry.path().string());
        }
    }
    return files;
}

int main() {
    std::string input_dir = "./../tests";
    std::string output_file_path = "./../output_cpp.txt";

    auto file_paths = get_files_in_directory(input_dir);
    std::ofstream output_file(output_file_path);
    LineReader reader(file_paths);

    // Start time measurement
    auto start_time = std::chrono::high_resolution_clock::now();

    std::thread writer_thread(write_lines, std::ref(reader), std::ref(output_file));

    writer_thread.join();

    // End time measurement
    auto end_time = std::chrono::high_resolution_clock::now();
    std::chrono::duration<double> elapsed = end_time - start_time;
    std::cout << "Total Elapsed Time: " << elapsed.count() << " seconds" << std::endl;

    return 0;
}



