using System;
using System.Collections.Generic;
using System.Linq;

class Utils {
    private static Random random = new Random();
    public static string GetEnvironmentVariable(string name, string defaultValue) => Environment.GetEnvironmentVariable(name) ?? defaultValue;

    public static string RandomString(int length) {
        const string pool = "abcdefghijklmnopqrstuvwxyz0123456789";
        var chars = Enumerable.Range(0, length)
            .Select(x => pool[random.Next(0, pool.Length)]);
        return new string(chars.ToArray());
    }

    public static bool AddOrUpdate<K, V>(IDictionary<K, V> map, K key, V value) {
        if (map.ContainsKey(key))
        {
            map[key] = value;
            return false;
        }
        else
        {
            map.Add(key, value);
            return true;
        }
    }

}