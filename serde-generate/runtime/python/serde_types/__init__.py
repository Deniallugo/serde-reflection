# Copyright (c) Facebook, Inc. and its affiliates
# SPDX-License-Identifier: MIT OR Apache-2.0

# pyre-ignore-all-errors

import numpy as np
from dataclasses import dataclass


@dataclass(init=False)
class uint128:
    high: np.uint64
    low: np.uint64

    def __init__(self, num):
        self.high = np.uint64(num >> 64)
        self.low = np.uint64(num & 0xFFFFFFFFFFFFFFFF)

    def __int__(self):
        return (int(self.high) << 64) | int(self.low)


@dataclass(init=False)
class int128:
    high: np.int64
    low: np.uint64

    def __init__(self, num):
        self.high = np.int64(num >> 64)
        self.low = np.uint64(num & 0xFFFFFFFFFFFFFFFF)

    def __int__(self):
        return (int(self.high) << 64) | int(self.low)


@dataclass(init=False)
class char:
    value: str

    def __init__(self, s):
        if len(s) != 1:
            raise ValueError("`char` expects a single unicode character")
        self.value = s


unit = type(None)

bool = np.bool
int8 = np.int8
int16 = np.int16
int32 = np.int32
int64 = np.int64

uint8 = np.uint8
uint16 = np.uint16
uint32 = np.uint32
uint64 = np.uint64

float32 = np.float32
float64 = np.float64
