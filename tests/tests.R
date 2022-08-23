rextendr::document()
#devtools::load_all(".")

audiotest:::test_in_R()


ArrayBase = audiotest:::load2("./test_files/mono.wav", mono = TRUE, offset = 0.0, duration = NA_real_) # decode audio into R Matrix
ArrayBase$print()
sr = audiotest::get_samplerate("./test_files/mono.wav")
audiotest:::play2(ArrayBase,sr)

# audio = audiotest::load("./test_files/mono.wav", mono = TRUE, offset = 0.0, duration = NA_real_) # decode audio into R Matrix
#  sr = audiotest::get_samplerate("./test_files/mono.wav")
#  audiotest::play(audio, sr)
# # sr
