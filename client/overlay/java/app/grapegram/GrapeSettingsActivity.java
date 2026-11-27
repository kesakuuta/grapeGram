package app.grapegram;

import android.content.Context;
import android.view.View;
import android.widget.FrameLayout;
import android.widget.Toast;

import org.telegram.messenger.AndroidUtilities;
import org.telegram.messenger.LocaleController;
import org.telegram.messenger.R;
import org.telegram.messenger.Utilities;
import org.telegram.ui.ActionBar.ActionBar;
import org.telegram.ui.ActionBar.AlertDialog;
import org.telegram.ui.ActionBar.BaseFragment;
import org.telegram.ui.ActionBar.Theme;
import org.telegram.ui.Components.IconBackgroundColors;
import org.telegram.ui.Components.LayoutHelper;
import org.telegram.ui.Components.UItem;
import org.telegram.ui.Components.UniversalAdapter;
import org.telegram.ui.Components.UniversalRecyclerView;
import org.telegram.ui.LauncherIconController;
import org.telegram.ui.LauncherIconController.LauncherIcon;
import org.telegram.ui.SettingsActivity.SettingCell;

import java.util.ArrayList;
import java.util.List;

import app.exteraless.ai.ui.AiSettingsActivity;
import app.exteraless.plugins.Plugin;
import app.exteraless.plugins.PluginsController;
import app.exteraless.plugins.ui.PluginActivityLogActivity;
import app.exteraless.plugins.ui.PluginPermissionsActivity;
import app.exteraless.plugins.ui.PluginsActivity;
import app.exteraless.settings.OpenExteraAppearanceActivity;
import app.exteraless.settings.OpenExteraAyuMomentsActivity;
import app.exteraless.settings.OpenExteraChatsActivity;

/** The single grapeGram settings hub, built from Telegram's own settings cells. */
public class GrapeSettingsActivity extends BaseFragment {

    private static final int ID_ICON = 1;
    private static final int ID_APPEARANCE = 2;
    private static final int ID_CHATS = 3;
    private static final int ID_AYU = 4;
    private static final int ID_AI = 5;
    private static final int ID_PLUGINS = 6;
    private static final int ID_PLUGIN_PERMISSIONS = 7;
    private static final int ID_PLUGIN_LOG = 8;
    private static final int ID_BADGES = 9;
    private static final int ID_ABOUT = 10;

    private UniversalRecyclerView listView;

    @Override
    public View createView(Context context) {
        setHasOwnBackground(true);

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

        FrameLayout contentView = new FrameLayout(context);
        contentView.setBackgroundColor(getThemedColor(Theme.key_windowBackgroundGray));

        listView = new UniversalRecyclerView(this, this::fillItems, this::onClick, null);
        listView.setSections();
        listView.adapter.setApplyBackground(false);
        listView.setSectionsDrawBackground(true);
        contentView.addView(listView, LayoutHelper.createFrame(LayoutHelper.MATCH_PARENT, LayoutHelper.MATCH_PARENT));
        actionBar.setAdaptiveBackground(listView);

        fragmentView = contentView;
        return fragmentView;
    }

    private void fillItems(ArrayList<UItem> items, UniversalAdapter adapter) {
        items.add(UItem.asHeader("grapeGram " + GrapeConfig.VERSION));
        items.add(SettingCell.Factory.of(
                ID_ICON, 0xFF7C4DFF, 0xFF5E35B1, R.drawable.grape_icon_original,
                "Иконка приложения", null, currentIconTitle()));
        items.add(UItem.asShadow(null));

        items.add(UItem.asHeader("Внешний вид"));
        items.add(row(ID_APPEARANCE, IconBackgroundColors.PURPLE, R.drawable.settings_account,
                "Оформление", "темы, анимации, Monet"));
        items.add(row(ID_CHATS, IconBackgroundColors.BLUE, R.drawable.settings_chat,
                "Чаты", "сообщения, папки, уведомления"));
        items.add(row(ID_AYU, IconBackgroundColors.ORANGE, R.drawable.settings_folders,
                "AyuMoments", "истории и статусы"));
        items.add(UItem.asShadow(null));

        items.add(UItem.asHeader("Интеллект и расширения"));
        items.add(row(ID_AI, IconBackgroundColors.CYAN, R.drawable.msg_bot,
                "AI", "провайдеры и модели"));
        items.add(row(ID_PLUGINS, IconBackgroundColors.GREEN, R.drawable.settings_premium,
                "Плагины", "установка, исходники, AI-анализ"));
        items.add(row(ID_PLUGIN_PERMISSIONS, IconBackgroundColors.RED, R.drawable.msg_permissions_solar,
                "Разрешения плагинов", "сеть, файлы, сообщения, хуки"));
        items.add(row(ID_PLUGIN_LOG, IconBackgroundColors.GRAY, R.drawable.settings_data,
                "Журнал активности", "события плагинов и отказы"));
        items.add(UItem.asShadow(null));

        items.add(UItem.asHeader("О приложении"));
        items.add(row(ID_BADGES, IconBackgroundColors.BLUE_DEEP, R.drawable.filled_profile_settings,
                "Бейджи", "подписанный список из GitHub"));
        items.add(row(ID_ABOUT, IconBackgroundColors.ORANGE_DEEP, R.drawable.settings_ask,
                "О grapeGram", "версия, авторы, лицензии"));
    }

    private static UItem row(int id, IconBackgroundColors colors, int icon, String title, String subtitle) {
        return SettingCell.Factory.of(id, colors.top, colors.bottom, icon, title, subtitle);
    }

    private void onClick(UItem item, View view, int position, float x, float y) {
        switch (item.id) {
            case ID_ICON:
                showIconPicker();
                break;
            case ID_APPEARANCE:
                presentFragment(new OpenExteraAppearanceActivity());
                break;
            case ID_CHATS:
                presentFragment(new OpenExteraChatsActivity());
                break;
            case ID_AYU:
                presentFragment(new OpenExteraAyuMomentsActivity());
                break;
            case ID_AI:
                presentFragment(new AiSettingsActivity());
                break;
            case ID_PLUGINS:
                presentFragment(new PluginsActivity());
                break;
            case ID_PLUGIN_PERMISSIONS:
                showPluginPicker(true);
                break;
            case ID_PLUGIN_LOG:
                showPluginPicker(false);
                break;
            case ID_BADGES:
                showBadges();
                break;
            case ID_ABOUT:
                showAbout();
                break;
            default:
                break;
        }
    }

    private String currentIconTitle() {
        for (LauncherIcon icon : LauncherIcon.values()) {
            if (icon.name().startsWith("GRAPE_") && LauncherIconController.isEnabled(icon)) {
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
        if (getParentActivity() == null) {
            return;
        }
        new AlertDialog.Builder(getParentActivity())
                .setTitle("Иконка приложения")
                .setItems(titles.toArray(new CharSequence[0]), (dialog, which) -> {
                    LauncherIconController.setIcon(icons.get(which));
                    toast("Иконка: " + titles.get(which));
                    if (listView != null && listView.adapter != null) {
                        listView.adapter.update(true);
                    }
                })
                .setNegativeButton(LocaleController.getString(R.string.Cancel), null)
                .show();
    }

    private void showPluginPicker(boolean permissions) {
        final Context context = getParentActivity();
        if (context == null) {
            return;
        }
        final List<Plugin> plugins = PluginsController.getInstance().getPluginsSnapshot();
        if (plugins == null || plugins.isEmpty()) {
            presentFragment(new PluginsActivity());
            return;
        }
        final CharSequence[] names = new CharSequence[plugins.size()];
        for (int i = 0; i < plugins.size(); i++) {
            names[i] = plugins.get(i).getDisplayName();
        }
        new AlertDialog.Builder(context)
                .setTitle(permissions ? "Разрешения плагинов" : "Журнал активности")
                .setItems(names, (dialog, which) -> {
                    final String pluginId = plugins.get(which).id;
                    if (permissions) {
                        presentFragment(new PluginPermissionsActivity(pluginId));
                    } else {
                        presentFragment(new PluginActivityLogActivity(pluginId));
                    }
                })
                .setNegativeButton(LocaleController.getString(R.string.Cancel), null)
                .show();
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

    private void toast(String text) {
        if (getParentActivity() != null) {
            Toast.makeText(getParentActivity(), text, Toast.LENGTH_SHORT).show();
        }
    }
}
