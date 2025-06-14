import os

import numpy as np
import pandas as pd
import soundfile as sf

from libfmp.c6 import compute_novelty_energy


if __name__ == '__main__':
    file_dir = os.path.dirname(os.path.realpath(__file__))
    x, Fs = sf.read(f'{file_dir}/../assets/LJ037-0171.wav')
    novelty_energy, Fs_feature = compute_novelty_energy(x, Fs=Fs, N=2048, H=128, gamma=10.0, norm=True)

    time = np.arange(len(novelty_energy)) * Fs_feature / Fs

    df = pd.DataFrame(zip(time, novelty_energy), columns=['time', 'novelty'])
    df.to_csv('LJ037-0171.csv', sep=',', float_format='%.5f', index=False)