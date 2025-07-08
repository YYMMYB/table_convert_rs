using Godot;
using System;
using System.Collections.Generic;
using System.IO;
using System.Text;
using FileAccess = Godot.FileAccess;

public class Cfg {
    public static Cfg I;
    public static cfg.Tables Tables => I.tb;

    static Cfg() {
        I = new Cfg();
        var access = new DataAccess();
        // 这里的路径换成你的路径
        var path = new DataPath("res://gen/cfg/data");
        I.tb = cfg.Tables.load(access, path);
    }

    public cfg.Tables tb;
}

public class DataAccess : IDataAccess {
    public string GetString(IDataPath path) {
        var p = (DataPath)path;
        var f = FileAccess.Open(p.path, FileAccess.ModeFlags.Read);
        var s = f.GetAsText();
        return s;
    }

    public IDataPath JoinPath(IDataPath path, string item) {
        return (path as DataPath).Join(item);
    }
}

public record class DataPath : IDataPath {
    public string path;

    public DataPath(string p) { path = p; }

    public DataPath Join(string s) {
        var npath = Path.Join(path, s);
        return new DataPath(npath);
    }
}