'''
An alternate algorithm for computing topplings.
'''
from collections import defaultdict
import subprocess
import json
import time
import os

from matplotlib.animation import FuncAnimation
import matplotlib.pyplot as plt
import seaborn as sns
import numpy as np


PATTERNS = ('+ x o o+ oo ox ++ +++ +_+ o++ o+++ o_+ o-+ o-+x o=+'
            ' +o xo +x x+ :: ;; Y A H sh')


def visualise_csv(sand_power, pattern, size=8, cmap="RdYlBu", save=False):
    '''
    The original algorithm wrote out csv files so this is provided as an
    interface to rendering them without computation.
    '''
    path = f'csvs/2_{sand_power}_{pattern}.csv'
    data = np.genfromtxt(path, delimiter=",")
    plt.figure(figsize=(size, size))
    plt.dpi = 300
    sns.heatmap(
        data, cbar=False, xticklabels=False, yticklabels=False, cmap=cmap
    )

    if save:
        plt.savefig(
            f'images/sandpile_2_{sand_power}_{pattern}.png',
            bbox_inches="tight"
        )


def rust_run(sand_power, pattern, force):
    '''
    Run the rust version of the algorithm for the requested starting sand
    and toppling pattern.

    If `force` is True then the generation code will be run even if a
    previous output file can be found in the `json` directory.
    '''
    path = f'json/{pattern}/2_{sand_power}_{pattern}.json'

    if force or not os.path.exists(path):
        start = time.time()
        proc = subprocess.run(
            f'./target/release/sandpiles {sand_power} {pattern}'.split(),
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE
        )
        print(proc.stdout.decode('utf-8'))
        print(proc.stderr.decode('utf-8'))
        print(f'Computation took {time.time() - start} seconds')

    with open(path, 'r') as f:
        return json.loads(f.read())


def visualise(sand_power, pattern, size=8, cmap="RdYlBu", save=False, force=False):
    '''Greys YlGnBu are also good'''
    data = rust_run(sand_power, pattern, force)

    grid = data['grid']
    plt.figure(figsize=(size, size))
    plt.dpi = 300
    sns.heatmap(
        grid, cbar=False, xticklabels=False, yticklabels=False, cmap=cmap
    )

    if save:
        plt.savefig(
            f'images/sandpile_2_{sand_power}_{pattern}.png',
            bbox_inches="tight"
        )

    return data


def animate_by_n(start_stop=(4, 22), pattern='+', fps=2, cmap="RdYlBu"):
    '''Make an animation of the growth of the pattern'''
    fig = plt.figure()
    fig.set_size_inches(5, 5, forward=True)

    data = rust_run(start_stop[0], pattern, force=False)
    ax = sns.heatmap(
        data, cbar=False, xticklabels=False,
        yticklabels=False, cmap=cmap
    )

    def init():
        ax = sns.heatmap(
            data, cbar=False, xticklabels=False,
            yticklabels=False, cmap=cmap
        )
        return ax,

    def animate(i, ax, fig):
        ax.cla()
        data = rust_run((start_stop[0] + i + 1), pattern, False)
        ax = sns.heatmap(
            data, cbar=False, xticklabels=False,
            yticklabels=False, cmap=cmap, ax=ax
        )
        return ax,

    anim = FuncAnimation(
        fig, animate, init_func=init,
        frames=(start_stop[1] - start_stop[0]),
        fargs=(ax, fig), repeat_delay=1000,
        interval=20,
    )

    fname = f'{pattern}_{start_stop[0]}_{start_stop[1]}.gif'
    anim.save(fname, writer="imagemagick", fps=fps, dpi=400)
    # Trim border
    subprocess.run(
        ["convert", fname, "-fuzz", "1%", "-trim",
         "+repage", "-delay", "500", fname])
    # pause at the end of the loop
    print("Now run the following to put back the delay:")
    print("convert {} \( +clone -set delay 500 \) +swap +delete {}".format(
        fname, fname))
    plt.show()


class Sandpile:
    '''
    Control class for toppling sandpiles.

    While not as efficient as the Rust version, there is more flexibility for
    tinkering with ideas in Python first before implementing in Rust.
    '''

    def __init__(self, sand_power, pattern):
        self.sand = 2 ** sand_power
        self.sand_power = sand_power
        self.topple_cells = self.parse(pattern)
        self.max_per_cell = len(self.topple_cells)
        self.max_dim = 1
        self.grid = defaultdict(int)
        self.grid[(0, 0)] = self.sand
        self.array = []

    @staticmethod
    def parse(pattern):
        '''
        Convert a string repr of a pattern to cell offsets.

        A pattern must be square and target cells are denoted with `x` while
        empty cells are denoted with `.`

        example: "+"
        .1.
        1.1  -> [(-1, 0), (1, 0), (0, 1), (0, -1)]
        .1.
        '''
        topple_cells = []
        rows = pattern.split()
        offset = len(rows) // 2  # midpoint

        for rix, row in enumerate(rows):
            for cix, cell in enumerate(row):
                if cell != '.':
                    for _ in range(int(cell)):
                        topple_cells.append((offset-cix, offset-rix))

        return topple_cells

    def topple(self):
        '''generate the fractal'''
        cell_max = self.sand + 1

        while cell_max >= self.max_per_cell:
            cell_max = 0
            new_sand = defaultdict(int)

            for (row, col), sand in self.grid.items():
                if sand >= self.max_per_cell:
                    per_cell = sand // self.max_per_cell
                    self.grid[(row, col)] %= self.max_per_cell

                    for (dx, dy) in self.topple_cells:
                        loc = (row + dx, col + dy)

                        if loc[0] > self.max_dim:
                            self.max_dim = loc[0]
                        if loc[1] > self.max_dim:
                            self.max_dim = loc[1]

                        new_sand[loc] += per_cell

            for cell, sand in new_sand.items():
                total = self.grid[cell] + sand
                self.grid[cell] = total
                if total > cell_max:
                    cell_max = total

        # We've reached steady state so flatten out the grid
        grid_size = self.max_dim * 2 + 1
        array = [[0 for i in range(grid_size)] for j in range(grid_size)]

        for (row, col), sand in self.grid.items():
            array[row+self.max_dim][col+self.max_dim] = sand

        self.array = array
        return self.array

    def double(self, plot=True):
        '''Double the sand on each current stable cell and topple'''
        for cell in self.grid:
            self.grid[cell] *= 2
        self.topple()
        if plot:
            self.plot()

    def resead(self, plot=True):
        '''Replace the starting sand and topple again'''
        self.grid[(0, 0)] = self.sand
        self.topple()
        if plot:
            self.plot()

    def replace_pattern(self, pattern):
        '''Swap out the toppling pattern for a new one'''
        self.topple_cells = self.parse(pattern)
        self.max_per_cell = len(self.topple_cells)

    def plot(self, size=8, cmap='RdYlBu'):
        '''Plot the current array'''
        plt.figure(figsize=(size, size))
        plt.dpi = 300
        sns.heatmap(
            self.array, cbar=False,
            xticklabels=False, yticklabels=False,
            cmap='RdYlBu'
        )
