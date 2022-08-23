rextendr::document()
devtools::load_all(".")

audio = audior::load("../../../test_files/mono.wav") # decode audio into R Matrix
sr = audior::get_samplerate("../../../test_files/mono.wav")
audior::play(audio, sr)
