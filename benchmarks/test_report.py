"""Checks for statistical mistakes that could change benchmark conclusions."""
import unittest
from report import host_description, summarize, startup_summary


class ProcessSummaryTests(unittest.TestCase):
    def fixture(self):
        samples = [[0, 1, 2], [3, 100, 101], [4, 5, 6]]
        return {'metadata': {'rounds': 3}, 'records': [
            {'kind': 'sample', 'runtime': 'test', 'case': 'example', 'round': round_index, 'nsPerOp': value}
            for round_index, values in enumerate(samples) for value in values]}

    def test_process_medians_are_not_pooled_batches(self):
        row = summarize(self.fixture())[('test', 'example')]
        # Pooled batch median would be 4, while the process medians are 1,100,5.
        self.assertEqual(row['median'], 5)
        self.assertEqual((row['q1'], row['q3']), (3, 52.5))

    def test_incomplete_measurements_are_refused(self):
        data = self.fixture(); data['records'].pop()
        with self.assertRaises(ValueError): summarize(data)
        data = self.fixture(); data['metadata']['rounds'] = 4
        with self.assertRaises(ValueError): summarize(data)

    def test_startup_modes_and_incomplete_probes(self):
        normal = [{'kind': 'startup-probe', 'mode': 'normal', 'phases': {'load': value}} for value in range(30)]
        control = [{'kind': 'startup-probe', 'mode': 'prewarm-web', 'phases': {'load': 1000}} for _ in range(30)]
        self.assertEqual(startup_summary(normal + control)['load'], 14.5)
        self.assertEqual(startup_summary(normal)['Instance'], 0)
        self.assertIsNone(startup_summary([]))
        with self.assertRaises(ValueError): startup_summary(normal[:-1])

    def test_host_label_uses_recorded_identity(self):
        meta = {'host': {'platform': 'example OS', 'machine': 'test arch', 'cpuAffinity': 19,
                         'midrEl1': '0x123', 'maxFrequencyKhz': '3980000', 'governor': 'performance'}}
        label = host_description(meta)
        for expected in ['example OS', 'test arch', '19', '0x123', '3980000', 'performance']:
            self.assertIn(expected, label)


if __name__ == '__main__':
    unittest.main()
