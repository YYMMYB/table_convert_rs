using System;
using System.Collections.Generic;
using System.Text.Json.Serialization;
using Cfg.Common;

namespace Cfg.Types;
public class Mod {
    public Dictionary<int, Cfg.Types.a.a_item> a;

    public Cfg.Types.命名空间.Mod 命名空间;

    public static Cfg.Types.Mod Load(IDataAccess access, IDataPath folder) {
        var tables = new Cfg.Types.Mod();

        // 数据表
        {
        var s = access.GetString(access.JoinPath(folder, "a.json"));
        tables.a = JsonSerializer.Deserialize<Dictionary<int, Cfg.Types.a.a_item>>(s, Cfg.Common.Utils.Options);
        }

        // 子模块
        tables.命名空间 = Cfg.Types.命名空间.Mod.Load(access, access.JoinPath(folder, "命名空间"));
        return tables;
    }
}
