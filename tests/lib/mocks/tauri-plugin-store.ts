/** Mock for @tauri-apps/plugin-store */

export class Store {
  private _data: Record<string, unknown> = {};

  static async load(_path: string): Promise<Store> {
    return new Store();
  }

  async get<T>(key: string): Promise<T | null> {
    return (this._data[key] as T) ?? null;
  }

  async set(key: string, value: unknown): Promise<void> {
    this._data[key] = value;
  }

  async delete(key: string): Promise<void> {
    delete this._data[key];
  }

  async clear(): Promise<void> {
    this._data = {};
  }

  async save(): Promise<void> {}

  async keys(): Promise<string[]> {
    return Object.keys(this._data);
  }

  async values(): Promise<unknown[]> {
    return Object.values(this._data);
  }

  async entries(): Promise<[string, unknown][]> {
    return Object.entries(this._data);
  }

  async length(): Promise<number> {
    return Object.keys(this._data).length;
  }

  async has(key: string): Promise<boolean> {
    return key in this._data;
  }

  async onChange(_callback: (key: string, value: unknown) => void): Promise<() => void> {
    return () => {};
  }
}
