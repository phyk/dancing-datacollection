import polars as pl
import os

def main():
    test_dirs = [d for d in os.listdir('data') if d.startswith('51-') or d.startswith('52-') or d.startswith('53-')]
    for comp_dir in test_dirs:
        part_path = os.path.join('data', comp_dir, 'participants.parquet')
        if os.path.exists(part_path):
            print(f'\nname_two values for {comp_dir}:')
            df = pl.read_parquet(part_path)
            if 'name_two' in df.columns:
                print('Unique name_two values:', df['name_two'].unique().to_list())
                print('Sample:')
                print(df[['name_one', 'name_two', 'club']])
            else:
                print('No name_two column found.')

if __name__ == '__main__':
    main() 