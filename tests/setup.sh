cd "$(dirname "$0")"
git clone https://github.com/SingleStepTests/ProcessorTests.git
mv ./ProcessorTests/65816/v1 ./cpu_cases
mv ./ProcessorTests/spc700/v1 ./apu_cases
rm -rf ./ProcessorTests
