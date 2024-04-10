import pytest
import subxtpy

# Test with small positive numbers
def test_sum_small_positive():
    result = subxtpy.sum_as_string(2, 3)
    assert result == "5", "Test with small positive numbers failed!"

# Test with zero and a positive number
def test_sum_zero_positive():
    result = subxtpy.sum_as_string(0, 5)
    assert result == "5", "Test with zero and a positive number failed!"

# Test with large positive numbers
def test_sum_large_positive():
    result = subxtpy.sum_as_string(123456, 654321)
    assert result == "777777", "Test with large positive numbers failed!"
