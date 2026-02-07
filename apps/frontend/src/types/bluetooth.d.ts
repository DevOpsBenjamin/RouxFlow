// Web Bluetooth API types
interface BluetoothDevice {
    id: string
    name?: string
    gatt?: BluetoothRemoteGATTServer
}

interface BluetoothRemoteGATTServer {
    device: BluetoothDevice
    connected: boolean
    connect(): Promise<BluetoothRemoteGATTServer>
    disconnect(): void
    getPrimaryService(service: string): Promise<BluetoothRemoteGATTService>
}

interface BluetoothRemoteGATTService {
    device: BluetoothDevice
    uuid: string
    isPrimary: boolean
    getCharacteristic(characteristic: string): Promise<BluetoothRemoteGATTCharacteristic>
}

interface BluetoothRemoteGATTCharacteristic {
    service: BluetoothRemoteGATTService
    uuid: string
    properties: BluetoothCharacteristicProperties
    value?: DataView
    addEventListener(type: string, listener: EventListener): void
    removeEventListener(type: string, listener: EventListener): void
    startNotifications(): Promise<BluetoothRemoteGATTCharacteristic>
    stopNotifications(): Promise<BluetoothRemoteGATTCharacteristic>
}

interface BluetoothCharacteristicProperties {
    broadcast: boolean
    read: boolean
    writeWithoutResponse: boolean
    write: boolean
    notify: boolean
    indicate: boolean
    authenticatedSignedWrites: boolean
    reliableWrite: boolean
    writableAuxiliaries: boolean
}
