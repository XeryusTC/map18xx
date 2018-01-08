# -*- coding: utf-8 -*-
import json
import toml
from unipath import Path

defdir = Path.cwd().child('tiledefs')
for orig_file in defdir.listdir('*.toml'):
    with open(orig_file) as orig_f:
        content = toml.load(orig_f)

        content['paths'] = []
        try:
            for path in content['path']:
                if "start" in path.keys():
                    path['start'] = {'Named': path['start']}
                elif "start_pos" in path.keys():
                    path['start'] = {'HexSpace': path['start_pos']}
                    del path['start_pos']

                if "end" in path.keys():
                    path['end'] = {'Named': path['end']}
                elif "end_pos" in path.keys():
                    path['end'] = {'HexSpace': path['end_pos']}
                    del path['end_pos']
                content['paths'].append(path)
            del content['path']
        except KeyError:
            pass # There are no paths defined
        if not content['paths']: # Remove if key was empty
            del content['paths']

        content['cities'] = []
        try:
            for city in content['city']:
                if "pos" in city.keys():
                    city['position'] = {'HexSpace': city['pos']}
                    del city['pos']
                elif "position" in city.keys():
                    city['position'] = {'Named': city['position']}
                else:
                    city['position'] = {'Named': 'C'}

                city['revenue_position'] = {'HexSpace': city['revenue_pos']}
                del city['revenue_pos']
                content['cities'].append(city)
            del content['city']
        except KeyError:
            pass # There are no cities defined
        if not content['cities']: # Remove if key was empty
            del content['cities']

        content['stops'] = []
        try:
            for stop in content['stop']:
                stop['revenue_angle'] *= -1
                stop['position'] = {'HexSpace': stop['position']}
                content['stops'].append(stop)
            del content['stop']
        except KeyError:
            pass # There are no stops defined
        if not content['stops']: # Remove if key was empty
            del content['stops']

        new_file = orig_file.parent.child(orig_file.stem + '.json')
        with open(new_file, 'w') as new_f:
            json.dump(content, new_f, indent='\t')
