from enum import Enum
import sys
import json
import telnetlib
from collections import namedtuple
from enum import Enum


State_ups = namedtuple('state_ups', ['temperature', 'main_voltage',
                        'charge_battery', 'capasity_battery', 'working_hours', 'load',
                        'model', 'date_battary_replacement', 'serial_number', 'status'
                        ],)


class Command_UPS(Enum):
    YES = b'Y'
    TEMPRETURE = b'C'
    MAINS_VOLTAGE = b'L'
    CHARGE_BATTERIES = b'f'
    CAPASITY_BATTERY = b'0'
    WORKING_HOURS = b'j'
    LOAD = b'P'
    MODEL = b'\x01' # CTRL + A
    DATA_BATTARY_REPLACEMENT = b'x'
    SERIAL_NUMBER = b'n'
    STATUS = 'Q'

def get_state_ups(host:str, password:str, username:str, port:int=2065,):
    try:
        telnet = _connect_UPS(host=host, port=port)
        auth = _check_auth(telnet=telnet)
        if auth:
            telnet = _authenticate_connection(telnet=telnet)
        values = _get_value_ups(telnet=telnet)
        values = _pars_values(values=values)
        state_ups = _valid_values(values=values)
    except Exception as err:
        raise err
    return state_ups

def _connect_UPS(host: str, port:int) -> telnetlib.Telnet:
    telnet = telnetlib.Telnet(host=host, port=port, timeout=2)
    return telnet

def _check_auth(telnet: telnetlib.Telnet) -> bool:
    if telnet.read_until(b'Username:', timeout=1):
        return True
    else:
        return False

def _get_value_ups(telnet:telnetlib.Telnet) -> dict:
    state_ups_dict = dict()
    for command in Command_UPS:
        telnet.write(command.value)
        value = telnet.read_until(b'\n', timeout=5)
        if not value:
            raise ValueError
        state_ups_dict[command.name] = value.decode('utf-8')
    telnet.close()
    return state_ups_dict
    
def _authenticate_connection(telnet: telnetlib.Telnet, username:str, password:str):
    telnet.write(str.encode(username) + b'\n')
    telnet.read_until(b'Password:', timeout=2)
    telnet.write(str.encode(password) + b'\n')
    return telnet

def _pars_values(values: dict) -> dict:
    for key, value in values.items():
        values[key] = value.strip('\r\n').strip(':')
    return values

def _valid_values(values: dict) -> State_ups:
    try:
        capasity_battery=float(values.get('CAPASITY_BATTERY'))
    except ValueError:
        capasity_battery = -1

    try:
        temperature = float(values.get('TEMPRETURE'))
    except ValueError:
        temperature = values.get('TEMPRETURE')

    state_ups = State_ups(temperature=temperature,
                            main_voltage=float( values.get('MAINS_VOLTAGE')),
                            charge_battery=values.get('CHARGE_BATTERIES'),
                            capasity_battery=capasity_battery,
                            working_hours=values.get('WORKING_HOURS'),
                            load=values.get('LOAD'),
                            model=values.get('MODEL'),
                            date_battary_replacement=values.get('DATA_BATTARY_REPLACEMENT'),
                            serial_number=values.get('SERIAL_NUMBER'),
                            status=values.get('STATUS'),

    )
    return state_ups

if __name__=="__main__":
    try:
        host = sys.argv[1]
        port = sys.argv[2]
        username = sys.argv[3]
        password = sys.argv[4]
        state = get_state_ups(host=host, port=port, username=username, password=password)
        state_json = json.dumps(state._asdict())
    except Exception:
        exit(10)
    
    