import matplotlib.pyplot as plt
import numpy as np


def main():
    with open("./benchmarkResultsLarger", "r") as f:
        lines_per_second_old = []
        for line in f:
            if line.startswith("Lines per second"):
                lines_per_second_old.append(float(line.split()[-1]))

    # Change to the new file
    with open("", "r") as f:
        lines_per_second_new = []
        for line in f:
            if line.startswith("Lines per second"):
                lines_per_second_new.append(float(line.split()[-1]))

    smoothed_old = [
        np.mean(lines_per_second_old[max(0, i - 1):min(i + 1, len(lines_per_second_old) - 1)]) for i in range(len(lines_per_second_old))
    ]
    smoothed_new = [
        np.mean(lines_per_second_new[max(0, i - 1):min(i + 1, len(lines_per_second_new) - 1)]) for i in range(len(lines_per_second_new))
    ]

    plt.xlabel('Number of Nodes')
    plt.ylabel('Lines Per Second')
    plt.title('Lines Per Second vs. Number of Nodes')

    plt.plot(range(1, len(lines_per_second_old) + 1), smoothed_old, c='blue', label="Old")
    plt.plot(range(1, len(lines_per_second_old) + 1), lines_per_second_old, "ro", label="Old")

    plt.plot(range(1, len(lines_per_second_new) + 1), smoothed_new, c='green', label="New")
    plt.plot(range(1, len(lines_per_second_new) + 1), lines_per_second_new, "go", label="New")

    plt.legend(loc='upper left')
    plt.show()


if __name__ == "__main__":
    main()

