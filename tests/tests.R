rextendr::document()
#devtools::load_all(".")

audiotest:::test_in_R()

# audio = audiotest::load("./test_files/mono.wav", mono = TRUE, offset = 0.0, duration = NA_real_) # decode audio into R Matrix
# sr = audiotest::get_samplerate("./test_files/mono.wav")
# audiotest::play(audio, sr)
# sr
