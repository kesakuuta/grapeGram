package app.grapegram;

import android.content.Context;
import android.content.DialogInterface;
import android.graphics.Typeface;
import android.util.TypedValue;
import android.view.Gravity;
import android.view.View;
import android.widget.LinearLayout;
import android.widget.ScrollView;
import android.widget.TextView;
import android.widget.Toast;

import org.telegram.messenger.AndroidUtilities;
import org.telegram.messenger.LocaleController;
import org.telegram.messenger.R;
import org.telegram.messenger.Utilities;
import org.telegram.ui.ActionBar.ActionBar;
import org.telegram.ui.ActionBar.AlertDialog;
import org.telegram.ui.ActionBar.BaseFragment;
import org.telegram.ui.ActionBar.Theme;
import org.telegram.ui.LauncherIconController;
import org.telegram.ui.LauncherIconController.LauncherIcon;

import java.util.ArrayList;
import java.util.List;

/** The single grapeGram settings hub. */
public class GrapeSettingsActivity extends BaseFragment {

    @Override
    public View createView(Context context) {
        actionBar.setBackButtonImage(R.drawable.ic_ab_back);
        actionBar.setAllowOverlayTitle(true);
        actionBar.setTitle("grapeGram");
        actionBar.setActionBarMenuOnItemClick(new ActionBar.ActionBarMenuOnItemClick() {
            @Override
            public void onItemClick(int id) {
                if (id == -1) {
                    finishFragment();
                }
            }
        });

        ScrollView scroll = new ScrollView(context);
        scroll.setBackgroundColor(Theme.getColor(Theme.key_windowBackgroundGray));
        LinearLayout list = new LinearLayout(context);
        list.setOrientation(LinearLayout.VERTICAL);
        list.setPadding(0, AndroidUtilities.dp(8), 0, AndroidUtilities.dp(16));
        scroll.addView(list, new ScrollView.LayoutParams(
                ScrollView.LayoutParams.MATCH_PARENT, ScrollView.LayoutParams.WRAP_CONTENT));

        addHeader(list, context, "grapeGram " + GrapeConfig.VERSION);

        addRow(list, context, "Иконка приложения", currentIconTitle(), v -> showIconPicker());
        addRow(list, context, "Настройки exteraGram / exteraless", "оформление, чаты, AI", v -> {
            try {
                presentFragment(new app.exteraless.settings.OpenExteraSettingsActivity());
            } catch (Throwable t) {
                toast("Недоступно: " + t.getClass().getSimpleName());
            }
        });
        addRow(list, context, "Плагины", "установка, разрешения, журнал", v -> openPlugins());
        addRow(list, context, "Бейджи", "подписанный список из GitHub", v -> showBadges());
        addRow(list, context, "О grapeGram", "версия, авторы, лицензии", v -> showAbout());

        fragmentView = scroll;
        return fragmentView;
    }

    private String currentIconTitle() {
        for (LauncherIcon icon : LauncherIcon.values()) {
            if (LauncherIconController.isEnabled(icon)) {
                return LocaleController.getString(icon.title);
            }
        }
        return "Оригинальная";
    }

    private void showIconPicker() {
        final List<LauncherIcon> icons = new ArrayList<>();
        final List<CharSequence> titles = new ArrayList<>();
        for (LauncherIcon icon : LauncherIcon.values()) {
            if (icon.name().startsWith("GRAPE_")) {
                icons.add(icon);
                titles.add(LocaleController.getString(icon.title));
            }
        }
        if (icons.isEmpty()) {
            toast("Иконки не найдены");
            return;
        }
        new AlertDialog.Builder(getParentActivity())
                .setTitle("Иконка приложения")
                .setItems(titles.toArray(new CharSequence[0]), (dialog, which) -> {
                    LauncherIconController.setIcon(icons.get(which));
                    toast("Иконка: " + titles.get(which));
                })
                .setNegativeButton(LocaleController.getString(R.string.Cancel), null)
                .show();
    }

    private void openPlugins() {
        try {
            Class<?> c = Class.forName("com.exteragram.messenger.plugins.ui.PluginsActivity");
            Object o = c.getDeclaredConstructor().newInstance();
            if (o instanceof BaseFragment) {
                presentFragment((BaseFragment) o);
                return;
            }
        } catch (Throwable ignored) {
        }
        toast("Экран плагинов недоступен в этой сборке");
    }

    private void showBadges() {
        final Context context = getParentActivity();
        if (context == null) {
            return;
        }
        final AlertDialog progress = new AlertDialog(context, AlertDialog.ALERT_TYPE_SPINNER);
        progress.setCanCancel(true);
        progress.show();
        Utilities.globalQueue.postRunnable(() -> {
            final GrapeBadges.Result result = GrapeBadges.load(context.getApplicationContext(), true);
            AndroidUtilities.runOnUIThread(() -> {
                try {
                    progress.dismiss();
                } catch (Throwable ignored) {
                }
                if (!result.ok) {
                    toast("Бейджи: " + result.error);
                    return;
                }
                StringBuilder sb = new StringBuilder();
                for (GrapeBadges.Entry e : result.entries) {
                    if (e.enabled) {
                        sb.append(e.userId).append(" — ").append(e.badge).append('\n');
                    }
                }
                if (sb.length() == 0) {
                    sb.append("Пока никому не выданы");
                }
                new AlertDialog.Builder(context)
                        .setTitle("Бейджи · v" + result.version)
                        .setMessage(sb.toString().trim())
                        .setPositiveButton(LocaleController.getString(R.string.OK), null)
                        .show();
            });
        });
    }

    private void showAbout() {
        new AlertDialog.Builder(getParentActivity())
                .setTitle("grapeGram " + GrapeConfig.VERSION)
                .setMessage("Клиент на базе Cherrygram, exteraGram и exteraless.\n"
                        + "Авторы upstream не поддерживают grapeGram.\n"
                        + "Лицензия GPL-3.0.")
                .setPositiveButton(LocaleController.getString(R.string.OK), null)
                .show();
    }

    private void addHeader(LinearLayout list, Context context, String text) {
        TextView tv = new TextView(context);
        tv.setText(text);
        tv.setTextSize(TypedValue.COMPLEX_UNIT_DIP, 13);
        tv.setTypeface(Typeface.DEFAULT_BOLD);
        tv.setTextColor(Theme.getColor(Theme.key_windowBackgroundWhiteGrayText2));
        tv.setPadding(AndroidUtilities.dp(18), AndroidUtilities.dp(12),
                AndroidUtilities.dp(18), AndroidUtilities.dp(6));
        list.addView(tv);
    }

    private void addRow(LinearLayout list, Context context, String title, String sub,
                        View.OnClickListener listener) {
        LinearLayout row = new LinearLayout(context);
        row.setOrientation(LinearLayout.VERTICAL);
        row.setBackgroundColor(Theme.getColor(Theme.key_windowBackgroundWhite));
        row.setPadding(AndroidUtilities.dp(18), AndroidUtilities.dp(12),
                AndroidUtilities.dp(18), AndroidUtilities.dp(12));
        row.setOnClickListener(listener);

        TextView t = new TextView(context);
        t.setText(title);
        t.setTextSize(TypedValue.COMPLEX_UNIT_DIP, 16);
        t.setTextColor(Theme.getColor(Theme.key_windowBackgroundWhiteBlackText));
        row.addView(t);

        if (sub != null && sub.length() > 0) {
            TextView s = new TextView(context);
            s.setText(sub);
            s.setTextSize(TypedValue.COMPLEX_UNIT_DIP, 13);
            s.setTextColor(Theme.getColor(Theme.key_windowBackgroundWhiteGrayText2));
            s.setPadding(0, AndroidUtilities.dp(2), 0, 0);
            row.addView(s);
        }

        LinearLayout.LayoutParams lp = new LinearLayout.LayoutParams(
                LinearLayout.LayoutParams.MATCH_PARENT, LinearLayout.LayoutParams.WRAP_CONTENT);
        lp.setMargins(0, AndroidUtilities.dp(1), 0, 0);
        list.addView(row, lp);
    }

    private void toast(String text) {
        if (getParentActivity() != null) {
            Toast.makeText(getParentActivity(), text, Toast.LENGTH_SHORT).show();
        }
    }
}
