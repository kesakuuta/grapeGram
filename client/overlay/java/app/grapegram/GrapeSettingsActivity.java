package app.grapegram;

import android.content.Context;
import android.graphics.Color;
import android.graphics.drawable.GradientDrawable;
import android.view.Gravity;
import android.view.View;
import android.widget.FrameLayout;
import android.widget.ImageView;
import android.widget.LinearLayout;
import android.widget.TextView;
import android.util.TypedValue;
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
import org.telegram.ui.SettingsActivity.SettingCell;
import org.telegram.ui.LiteModeSettingsActivity;

import java.util.ArrayList;

import app.exteraless.ai.ui.AiSettingsActivity;
import app.exteraless.plugins.ui.PluginsActivity;
import app.exteraless.settings.OpenExteraAppearanceActivity;
import app.exteraless.settings.OpenExteraAyuMomentsActivity;
import app.exteraless.settings.OpenExteraChatsActivity;
import app.exteraless.settings.OpenExteraCloudActivity;
import app.exteraless.settings.OpenExteraGeneralActivity;
import app.exteraless.settings.OpenExteraOtherActivity;

/** The single grapeGram settings hub, built from Telegram's own settings cells. */
public class GrapeSettingsActivity extends BaseFragment {

    private static final int ID_APPEARANCE = 1;
    private static final int ID_CHATS = 2;
    private static final int ID_AYU = 3;
    private static final int ID_AI = 4;
    private static final int ID_PLUGINS = 5;
    private static final int ID_BADGES = 6;
    private static final int ID_CREDITS = 7;
    private static final int ID_OTHER = 9;
    private static final int ID_CLOUD = 10;
    private static final int ID_ANIMATIONS = 11;
    private static final int ID_PERFORMANCE = 12;
    private static final int ID_APP_ICON = 13;

    private UniversalRecyclerView listView;
    private ImageView heroLogo;

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

    @Override
    public void onResume() {
        super.onResume();
        if (heroLogo != null && getParentActivity() != null) {
            heroLogo.setImageResource(GrapeIconResolver.getSelectedIcon(getParentActivity()));
        }
    }

    private void fillItems(ArrayList<UItem> items, UniversalAdapter adapter) {
        Context context = getContext();
        if (context != null) {
            items.add(UItem.asCustom(createHero(context), 136));
            items.add(UItem.asShadow(null));
        }

        items.add(UItem.asHeader("Персонализация"));
        items.add(row(ID_APPEARANCE, IconBackgroundColors.PURPLE, R.drawable.msg_theme,
                "Внешний вид", "темы, цвета, Monet и элементы интерфейса"));
        items.add(row(ID_ANIMATIONS, IconBackgroundColors.BLUE, R.drawable.msg2_animations,
                "Анимации и плавность", "системные эффекты, анимации и режим экономии"));
        items.add(row(ID_CHATS, IconBackgroundColors.CYAN, R.drawable.settings_chat,
                "Настройки чатов", "сообщения, папки и поведение диалогов"));
        items.add(row(ID_APP_ICON, IconBackgroundColors.PURPLE, R.drawable.grape_icon_original,
                "Иконка приложения", "выбрать одну из иконок grapeGram"));
        items.add(row(ID_AYU, IconBackgroundColors.ORANGE_DEEP, R.drawable.settings_folders,
                "AyuMoments", "истории и статусы"));
        items.add(UItem.asShadow(null));

        items.add(UItem.asHeader("Производительность и функции"));
        items.add(row(ID_PERFORMANCE, IconBackgroundColors.GREEN_DEEP, R.drawable.msg_speed,
                "Производительность", "скорость сети, загрузок и оптимизация батареи"));
        items.add(row(ID_OTHER, IconBackgroundColors.GRAY, R.drawable.msg_fave,
                "Дополнительные функции", "эксперименты и резервные копии"));
        items.add(row(ID_AI, IconBackgroundColors.CYAN, R.drawable.msg_bot,
                "AI-помощник", "провайдеры, API и модели"));
        items.add(UItem.asShadow(null));

        items.add(UItem.asHeader("Плагины"));
        items.add(row(ID_PLUGINS, IconBackgroundColors.GREEN, R.drawable.msg_plugins,
                "Менеджер плагинов", "установка, просмотр исходников и AI-разбор кода"));
        items.add(UItem.asShadow(null));

        items.add(UItem.asHeader("Аккаунт и grapeGram"));
        items.add(row(ID_BADGES, IconBackgroundColors.BLUE_DEEP, R.drawable.filled_profile_settings,
                "Бейджи grapeGram", "проверка подписанного списка"));
        items.add(row(ID_CLOUD, IconBackgroundColors.CYAN, R.drawable.cloud_sync,
                "Резервная копия", "сохранение и восстановление настроек"));
        items.add(row(ID_CREDITS, IconBackgroundColors.ORANGE_DEEP, R.drawable.settings_ask,
                "Авторы и благодарности", "проекты, сообщества и лицензии"));
    }

    private View createHero(Context context) {
        LinearLayout card = new LinearLayout(context);
        card.setOrientation(LinearLayout.HORIZONTAL);
        card.setGravity(Gravity.CENTER_VERTICAL);
        card.setPadding(AndroidUtilities.dp(20), AndroidUtilities.dp(16),
                AndroidUtilities.dp(20), AndroidUtilities.dp(16));
        GradientDrawable background = new GradientDrawable(
                GradientDrawable.Orientation.TL_BR,
                new int[]{0xFF503B9A, 0xFF7956C8, 0xFF9A69D8});
        background.setCornerRadius(AndroidUtilities.dp(22));
        card.setBackground(background);
        card.setElevation(AndroidUtilities.dp(2));

        ImageView logo = new ImageView(context);
        heroLogo = logo;
        logo.setImageResource(GrapeIconResolver.getSelectedIcon(context));
        logo.setScaleType(ImageView.ScaleType.FIT_CENTER);
        card.addView(logo, new LinearLayout.LayoutParams(
                AndroidUtilities.dp(68), AndroidUtilities.dp(68)));

        LinearLayout labels = new LinearLayout(context);
        labels.setOrientation(LinearLayout.VERTICAL);
        labels.setGravity(Gravity.CENTER_VERTICAL);
        LinearLayout.LayoutParams labelsParams = new LinearLayout.LayoutParams(
                0, LinearLayout.LayoutParams.WRAP_CONTENT, 1f);
        labelsParams.leftMargin = AndroidUtilities.dp(16);
        card.addView(labels, labelsParams);

        TextView title = new TextView(context);
        title.setText("grapeGram");
        title.setTextColor(Color.WHITE);
        title.setTypeface(AndroidUtilities.bold());
        title.setTextSize(TypedValue.COMPLEX_UNIT_DIP, 23);
        labels.addView(title, new LinearLayout.LayoutParams(
                LinearLayout.LayoutParams.WRAP_CONTENT, LinearLayout.LayoutParams.WRAP_CONTENT));

        TextView subtitle = new TextView(context);
        subtitle.setText("Настройки клиента");
        subtitle.setTextColor(0xEFFFFFFF);
        subtitle.setTextSize(TypedValue.COMPLEX_UNIT_DIP, 14);
        LinearLayout.LayoutParams subtitleParams = new LinearLayout.LayoutParams(
                LinearLayout.LayoutParams.WRAP_CONTENT, LinearLayout.LayoutParams.WRAP_CONTENT);
        subtitleParams.topMargin = AndroidUtilities.dp(3);
        labels.addView(subtitle, subtitleParams);

        TextView version = new TextView(context);
        version.setText("Версия " + GrapeConfig.VERSION + "  ·  Android");
        version.setTextColor(0xBFFFFFFF);
        version.setTextSize(TypedValue.COMPLEX_UNIT_DIP, 11);
        LinearLayout.LayoutParams versionParams = new LinearLayout.LayoutParams(
                LinearLayout.LayoutParams.WRAP_CONTENT, LinearLayout.LayoutParams.WRAP_CONTENT);
        versionParams.topMargin = AndroidUtilities.dp(7);
        labels.addView(version, versionParams);

        FrameLayout inset = new FrameLayout(context);
        inset.setClipToPadding(false);
        inset.setPadding(AndroidUtilities.dp(16), AndroidUtilities.dp(6),
                AndroidUtilities.dp(16), AndroidUtilities.dp(6));
        inset.addView(card, new FrameLayout.LayoutParams(
                FrameLayout.LayoutParams.MATCH_PARENT, FrameLayout.LayoutParams.MATCH_PARENT));
        return inset;
    }

    private static UItem row(int id, IconBackgroundColors colors, int icon, String title, String subtitle) {
        return SettingCell.Factory.of(id, colors.top, colors.bottom, icon, title, subtitle);
    }

    private void onClick(UItem item, View view, int position, float x, float y) {
        switch (item.id) {
            case ID_APPEARANCE:
                presentFragment(new OpenExteraAppearanceActivity());
                break;
            case ID_ANIMATIONS:
                presentFragment(new LiteModeSettingsActivity());
                break;
            case ID_PERFORMANCE:
                presentFragment(new OpenExteraGeneralActivity());
                break;
            case ID_CHATS:
                presentFragment(new OpenExteraChatsActivity());
                break;
            case ID_APP_ICON:
                presentFragment(new app.exteraless.appicons.AppIconsActivity());
                break;
            case ID_AYU:
                presentFragment(new OpenExteraAyuMomentsActivity());
                break;
            case ID_OTHER:
                presentFragment(new OpenExteraOtherActivity());
                break;
            case ID_AI:
                presentFragment(new AiSettingsActivity());
                break;
            case ID_PLUGINS:
                presentFragment(new PluginsActivity());
                break;
            case ID_BADGES:
                showBadges();
                break;
            case ID_CLOUD:
                presentFragment(new OpenExteraCloudActivity());
                break;
            case ID_CREDITS:
                presentFragment(new GrapeCreditsActivity());
                break;
            default:
                break;
        }
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
                GrapeBadgeBridge.syncCachedAsync(context.getApplicationContext());
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

    private void toast(String text) {
        if (getParentActivity() != null) {
            Toast.makeText(getParentActivity(), text, Toast.LENGTH_SHORT).show();
        }
    }
}
