import pandas as pd
import csv

# Read the CSV file
df = pd.read_csv('DeSales G1.csv')

# Initialize variables to keep track of runners and score differential
runners_on = 0
score_diff = 0

# Initialize variable to store the last transformed row
last_transformed_row = None

# List to store transformed data
transformed_data = []

# Iterate through the DataFrame
for index, row in df.iterrows():
    # Update runners on base
    if row['PlayResult']:
        runners_on -= row["RunsScored"]
    if row['PlayResult'] == 'Single' or row['PlayResult'] == 'Double' or row['PlayResult'] == 'Triple':
        runners_on += 1
    if row["Outs"] + row["OutsOnPlay"] == 3 or row["PlayResult"] == "HomeRun":
        runners_on = 0

    # Update score differential
    if row['RunsScored'] and row['BatterTeam'] == 'DES_UNI':
        score_diff += row['RunsScored']
    elif row['RunsScored'] and row['BatterTeam'] == 'LEB_VAL':
        score_diff -= row['RunsScored']

    # Format the data
    transformed_row = [f'{"V" if row["Top/Bottom"] == "Top" else "H"}', row["Inning"], row["Outs"], runners_on,
                       score_diff]

    # Append to the list if it has changed since the last one
    if transformed_row != last_transformed_row:
        transformed_data.append(transformed_row)
        last_transformed_row = transformed_row

# Convert the list to a DataFrame
transformed_df = pd.DataFrame(transformed_data, columns=['homeOrVisitor', 'inning', 'outs', 'runners', 'scoreDiff'])

# Custom quoting behavior dictionary
quoting_dict = {0: csv.QUOTE_NONNUMERIC}

# Write DataFrame to CSV with custom quoting behavior
transformed_df.to_csv('transformed_data.csv', index=False, quoting=csv.QUOTE_NONNUMERIC)
